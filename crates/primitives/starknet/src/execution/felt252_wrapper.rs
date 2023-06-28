//! # Felt252 - FieldElement wrapper.
//!
//! Starknet base type is a [`FieldElement`] from starknet-ff crate.
//! Substrate primitives are passed back and forth between client
//! and runtime using SCALE encoding: https://docs.substrate.io/reference/scale-codec/.
//!
//! The [`Felt252Wrapper`] implements the traits for SCALE encoding, and wrap
//! the [`FieldElement`] type from starknet-ff.

use alloc::string::String;

use cairo_vm::felt::Felt252;
use scale_codec::{Decode, Encode, EncodeLike, Error, Input, MaxEncodedLen, Output};
use scale_info::build::Fields;
use scale_info::{Path, Type, TypeInfo};
use sp_core::{H256, U256};
use starknet_api::hash::StarkFelt;
use starknet_ff::{FieldElement, FromByteSliceError, FromStrError};
use thiserror_no_std::Error;

///
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Hash, Eq, Copy, serde::Serialize, serde::Deserialize)]
//#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Felt252Wrapper(pub FieldElement);

impl Felt252Wrapper {
    /// Field252 constant that's equal to 0
    pub const ZERO: Self = Self(FieldElement::ZERO);
    /// Field252 constant that's equal to 1
    pub const ONE: Self = Self(FieldElement::ONE);
    /// Field252 constant that's equal to 2
    pub const TWO: Self = Self(FieldElement::TWO);
    /// Field252 constant that's equal to 3
    pub const THREE: Self = Self(FieldElement::THREE);
    /// Field252 constant that's equal to 2^251 + 17 * 2^192
    pub const MAX: Self = Self(FieldElement::MAX);

    /// Initializes from a hex string.
    ///
    /// # Arguments
    ///
    /// * `value` - A valid hex string prefixed with '0x`, with or without padding zeros.
    ///
    /// # Errors
    ///
    /// Hex string may contain a value that overflows felt252.
    /// If there if an overflow or invalid hex string,
    /// returns [`Felt252WrapperError`].
    pub fn from_hex_be(value: &str) -> Result<Self, Felt252WrapperError> {
        let fe = FieldElement::from_hex_be(value)?;
        Ok(Self(fe))
    }

    /// Initializes from a decimal string.
    ///
    /// # Arguments
    ///
    /// * `value` - A valid decimal string.
    ///
    /// # Errors
    ///
    /// Decimal string may contain a value that overflows felt252.
    /// If there if an overflow or invalid character in the string,
    /// returns [`Felt252WrapperError`].
    pub fn from_dec_str(value: &str) -> Result<Self, Felt252WrapperError> {
        let fe = FieldElement::from_dec_str(value)?;
        Ok(Self(fe))
    }
}

#[cfg(feature = "std")]
impl Felt252Wrapper {
    /// Decodes the bytes representation in utf-8
    ///
    /// # Errors
    ///
    /// If the bytes are not valid utf-8, returns [`Felt252WrapperError`].
    pub fn from_utf8(&self) -> Result<String, Felt252WrapperError> {
        let s =
            std::str::from_utf8(&self.0.to_bytes_be()).map_err(|_| Felt252WrapperError::InvalidCharacter)?.to_string();
        Ok(s.trim_start_matches('\0').to_string())
    }
}

impl Default for Felt252Wrapper {
    fn default() -> Self {
        Self(FieldElement::ZERO)
    }
}

/// Array of bytes from [`Felt252Wrapper`].
impl From<Felt252Wrapper> for [u8; 32] {
    fn from(felt: Felt252Wrapper) -> Self {
        felt.0.to_bytes_be()
    }
}

/// [`Felt252Wrapper`] from bytes.
/// Overflow may occur and return [`Felt252WrapperError::OutOfRange`].
impl TryFrom<&[u8; 32]> for Felt252Wrapper {
    type Error = Felt252WrapperError;

    fn try_from(bytes: &[u8; 32]) -> Result<Self, Felt252WrapperError> {
        match FieldElement::from_bytes_be(bytes) {
            Ok(ff) => Ok(Self(ff)),
            Err(_) => Err(Felt252WrapperError::FromArrayError),
        }
    }
}

/// [`Felt252Wrapper`] from bytes.
/// Overflow may occur and return [`Felt252WrapperError::OutOfRange`].
impl TryFrom<&[u8]> for Felt252Wrapper {
    type Error = Felt252WrapperError;

    fn try_from(bytes: &[u8]) -> Result<Self, Felt252WrapperError> {
        match FieldElement::from_byte_slice_be(bytes) {
            Ok(ff) => Ok(Self(ff)),
            Err(e) => match e {
                FromByteSliceError::InvalidLength => Err(Felt252WrapperError::InvalidLength),
                FromByteSliceError::OutOfRange => Err(Felt252WrapperError::OutOfRange),
            },
        }
    }
}

/// [`u64`] to [`Felt252Wrapper`].
impl From<u64> for Felt252Wrapper {
    fn from(value: u64) -> Self {
        Self(FieldElement::from(value))
    }
}

/// [`u32`] to [`Felt252Wrapper`].
impl From<u32> for Felt252Wrapper {
    fn from(value: u32) -> Self {
        Self(FieldElement::from(value))
    }
}

/// [`u8`] to [`Felt252Wrapper`].
impl From<u8> for Felt252Wrapper {
    fn from(value: u8) -> Self {
        Self(FieldElement::from(value))
    }
}

/// [`u128`] to [`Felt252Wrapper`].
impl From<u128> for Felt252Wrapper {
    fn from(value: u128) -> Self {
        Felt252Wrapper::try_from(U256::from(value)).unwrap()
    }
}

/// [`Felt252Wrapper`] to [`u64`].
/// Overflow may occur and return [`Felt252WrapperError::ValueTooLarge`].
impl TryFrom<Felt252Wrapper> for u64 {
    type Error = Felt252WrapperError;

    fn try_from(value: Felt252Wrapper) -> Result<Self, Self::Error> {
        u64::try_from(value.0).map_err(|_| Felt252WrapperError::ValueTooLarge)
    }
}

/// [`Felt252Wrapper`] to [`U256`].
impl From<Felt252Wrapper> for U256 {
    fn from(felt: Felt252Wrapper) -> Self {
        U256::from_big_endian(&felt.0.to_bytes_be())
    }
}

/// [`Felt252Wrapper`] from [`U256`].
/// Overflow may occur and return [`Felt252WrapperError::OutOfRange`].
impl TryFrom<U256> for Felt252Wrapper {
    type Error = Felt252WrapperError;

    fn try_from(u256: U256) -> Result<Self, Felt252WrapperError> {
        let mut buf: [u8; 32] = [0; 32];
        u256.to_big_endian(&mut buf);

        Felt252Wrapper::try_from(&buf)
    }
}

/// [`Felt252Wrapper`] from [`H256`].
/// Overflow may occur and return [`Felt252WrapperError::OutOfRange`].
impl TryFrom<H256> for Felt252Wrapper {
    type Error = Felt252WrapperError;

    fn try_from(h: H256) -> Result<Self, Felt252WrapperError> {
        Felt252Wrapper::try_from(h.as_bytes())
    }
}

/// [`Felt252Wrapper`] to [`H256`].
impl From<Felt252Wrapper> for H256 {
    fn from(felt: Felt252Wrapper) -> Self {
        let buf: [u8; 32] = felt.into();
        H256::from_slice(&buf)
    }
}

/// [`Felt252Wrapper`] from [`FieldElement`].
impl From<FieldElement> for Felt252Wrapper {
    fn from(ff: FieldElement) -> Self {
        Self(ff)
    }
}

/// [`Felt252Wrapper`] to [`FieldElement`].
impl From<Felt252Wrapper> for FieldElement {
    fn from(ff: Felt252Wrapper) -> Self {
        ff.0
    }
}

/// [`Felt252Wrapper`] from [`Felt252`].
impl From<Felt252> for Felt252Wrapper {
    fn from(value: Felt252) -> Self {
        Felt252Wrapper::try_from(&value.to_be_bytes()).unwrap()
    }
}

/// [`Felt252Wrapper`] to [`Felt252`].
impl From<Felt252Wrapper> for Felt252 {
    fn from(felt: Felt252Wrapper) -> Self {
        let buf: [u8; 32] = felt.into();
        Felt252::from_bytes_be(&buf)
    }
}

/// [`Felt252Wrapper`] from [`StarkFelt`].
impl From<StarkFelt> for Felt252Wrapper {
    fn from(value: StarkFelt) -> Self {
        Felt252Wrapper::try_from(value.bytes()).unwrap()
    }
}

/// [`Felt252Wrapper`] to [`StarkFelt`].
impl From<Felt252Wrapper> for StarkFelt {
    fn from(felt: Felt252Wrapper) -> Self {
        let buf: [u8; 32] = felt.into();
        StarkFelt::new(buf).unwrap()
    }
}

/// SCALE trait.
impl Encode for Felt252Wrapper {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(&self.0.to_bytes_be());
    }
}

/// SCALE trait.
impl EncodeLike for Felt252Wrapper {}

/// SCALE trait.
impl MaxEncodedLen for Felt252Wrapper {
    fn max_encoded_len() -> usize {
        32
    }
}

/// SCALE trait.
impl Decode for Felt252Wrapper {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let mut buf: [u8; 32] = [0; 32];
        input.read(&mut buf)?;

        match Felt252Wrapper::try_from(&buf) {
            Ok(felt) => Ok(felt),
            Err(e) => Err(Error::from("Can't get FieldElement from input buffer.").chain(hex::encode(buf)).chain(e)),
        }
    }
}

/// SCALE trait.
impl TypeInfo for Felt252Wrapper {
    type Identity = Self;

    // The type info is saying that the field element must be seen as an
    // array of bytes.
    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("Felt252Wrapper", module_path!()))
            .composite(Fields::unnamed().field(|f| f.ty::<[u8; 32]>().type_name("FieldElement")))
    }
}

#[derive(Debug, PartialEq, Error)]
/// Error related to Felt252Wrapper.
pub enum Felt252WrapperError {
    /// Conversion from byte array has failed.
    #[error("input array invalid")]
    FromArrayError,
    /// Provided byte array has incorrect lengths.
    #[error("invalid length")]
    InvalidLength,
    /// Invalid character in hex string.
    #[error("invalid character")]
    InvalidCharacter,
    /// Value is too large for FieldElement (felt252).
    #[error("number out of range")]
    OutOfRange,
    /// Value is too large to fit into target type.
    #[error("felt252 value too large")]
    ValueTooLarge,
}

use alloc::borrow::Cow;

impl From<Felt252WrapperError> for Cow<'static, str> {
    fn from(err: Felt252WrapperError) -> Self {
        match err {
            Felt252WrapperError::FromArrayError => Cow::Borrowed("input array invalid"),
            Felt252WrapperError::InvalidCharacter => Cow::Borrowed("invalid character"),
            Felt252WrapperError::OutOfRange => Cow::Borrowed("number out of range"),
            Felt252WrapperError::InvalidLength => Cow::Borrowed("invalid length"),
            Felt252WrapperError::ValueTooLarge => Cow::Borrowed("felt252 value too large"),
        }
    }
}

impl From<Felt252WrapperError> for String {
    fn from(felt_error: Felt252WrapperError) -> Self {
        match felt_error {
            Felt252WrapperError::FromArrayError => String::from("input array invalid"),
            Felt252WrapperError::InvalidCharacter => String::from("invalid character"),
            Felt252WrapperError::OutOfRange => String::from("number out of range"),
            Felt252WrapperError::InvalidLength => String::from("invalid length"),
            Felt252WrapperError::ValueTooLarge => String::from("felt252 value too large"),
        }
    }
}

impl From<FromStrError> for Felt252WrapperError {
    fn from(err: FromStrError) -> Self {
        match err {
            FromStrError::InvalidCharacter => Self::InvalidCharacter,
            FromStrError::OutOfRange => Self::OutOfRange,
        }
    }
}

#[cfg(test)]
mod felt252_wrapper_tests {

    use super::*;

    #[test]
    fn default_value() {
        assert_eq!(Felt252Wrapper::default(), Felt252Wrapper(FieldElement::ZERO));
    }

    #[test]
    fn from_hex_be() {
        Felt252Wrapper::from_hex_be("0x0").unwrap();
        Felt252Wrapper::from_hex_be("0x123456").unwrap();
        Felt252Wrapper::from_hex_be("0x01dbc98a49405a81587a9608c9c0b9fd51d65b55b0bf428bad499ab76c7b46d1").unwrap();

        let mut felt = Felt252Wrapper::from_hex_be(
            "0x01dbc98a49405a81587a9608c9c0b9fd51d65b55b0bf428bad499ab76c7b46d19722957295752795927529759275927572",
        );
        assert_eq!(felt, Err(Felt252WrapperError::OutOfRange));

        felt = Felt252Wrapper::from_hex_be("0xföífg¤gí’¤");
        assert_eq!(felt, Err(Felt252WrapperError::InvalidCharacter));
    }

    #[test]
    fn from_dec_str() {
        let f = Felt252Wrapper::from_dec_str("1").unwrap();
        assert_eq!(f, Felt252Wrapper::ONE);

        Felt252Wrapper::from_dec_str("1991991").unwrap();
    }

    #[test]
    fn felt252_from_fieldelement_twoway() {
        let fe = FieldElement::TWO;
        let felt: Felt252Wrapper = fe.into();
        assert_eq!(felt, Felt252Wrapper(FieldElement::TWO));

        let felt2 = Felt252Wrapper::from(fe);
        assert_eq!(felt2, Felt252Wrapper(FieldElement::TWO));

        let felt3 = Felt252Wrapper(FieldElement::THREE);
        let fe3: FieldElement = felt3.into();
        assert_eq!(fe3, FieldElement::THREE);
        assert_eq!(FieldElement::from(felt3), FieldElement::THREE);
    }

    #[test]
    fn felt252_from_u256_twoway() {
        let u = U256::from_little_endian(&[1]);
        let felt = Felt252Wrapper::try_from(u);
        assert_eq!(felt, Ok(Felt252Wrapper::ONE));

        let felt2 = Felt252Wrapper::TWO;
        let u2: U256 = felt2.into();
        assert_eq!(U256::from_little_endian(&[2]), u2);
    }

    #[test]
    fn felt252_from_h256_twoway() {
        let h = H256::from_low_u64_be(1);
        let felt: Felt252Wrapper = h.try_into().unwrap();
        assert_eq!(felt, Felt252Wrapper::ONE);

        let felt2 = Felt252Wrapper::TWO;
        let h2: H256 = felt2.into();
        let h2_expected = H256::from_low_u64_be(2);
        assert_eq!(h2, h2_expected);
    }

    #[test]
    fn encode_decode_scale() {
        let felt = Felt252Wrapper::ONE;
        let encoded = felt.encode();
        let decoded = Felt252Wrapper::decode(&mut &encoded[..]);
        assert_eq!(decoded, Ok(Felt252Wrapper(FieldElement::ONE)));

        let felt = Felt252Wrapper::from_hex_be("0x1234").unwrap();
        let encoded = felt.encode();
        let decoded = Felt252Wrapper::decode(&mut &encoded[..]);
        assert_eq!(felt, decoded.unwrap());
    }

    #[test]
    fn vec_encode_decode_scale() {
        let input = vec![
            Felt252Wrapper::ONE,
            Felt252Wrapper::TWO,
            Felt252Wrapper::from_dec_str("1000000000").unwrap(),
            Felt252Wrapper::MAX,
        ];
        let encoded = input.encode();
        let decoded = Vec::<Felt252Wrapper>::decode(&mut &encoded[..]);
        assert_eq!(decoded, Ok(input));
    }

    #[test]
    fn felt252_from_primitives() {
        let felt_u64 = Felt252Wrapper::from(4_294_967_296u64);
        assert_eq!(felt_u64, Felt252Wrapper::from_dec_str("4294967296").unwrap());

        let felt_u128 = Felt252Wrapper::from(18_446_744_073_709_551_616u128);
        assert_eq!(felt_u128, Felt252Wrapper::from_dec_str("18446744073709551616").unwrap());
    }

    #[test]
    fn primitives_try_from_felt252() {
        let felt_u64 = Felt252Wrapper::from(4_294_967_296u64);
        assert_eq!(TryInto::<u64>::try_into(felt_u64).unwrap(), 4_294_967_296u64);
    }

    #[test]
    fn decode_utf8() {
        let felt = Felt252Wrapper::from_hex_be("0x534e5f474f45524c49").unwrap();
        assert_eq!(felt.from_utf8().unwrap(), "SN_GOERLI".to_string());
    }
}
