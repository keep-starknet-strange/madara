use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Eq;
use core::fmt::Debug;
use core::hash::Hash;

use cairo_vm::felt::Felt252;
use cairo_vm::serde::deserialize_program::{
    ApTracking, Attribute, BuiltinName, FlowTrackingData, HintLocation, HintParams, Identifier, InputFile,
    InstructionLocation, Location, Member, OffsetValue, Reference, ReferenceManager, ValueAddress,
};
use cairo_vm::types::instruction::{ApUpdate, FpUpdate, Instruction, Op1Addr, Opcode, PcUpdate, Register, Res};
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::{MaybeRelocatable, Relocatable};
use derive_more::Constructor;
use frame_support::{BoundedBTreeMap, BoundedVec};
use sp_core::{ConstU32, Get, H256, U256};
use starknet_api::stdlib::collections::HashMap;
use starknet_ff::FieldElement;

#[cfg(feature = "std")]
use super::{
    deserialize_bounded_btreemap, deserialize_option_bounded_btreemap, serialize_bounded_btreemap,
    serialize_option_bounded_btreemap,
};

type MaxConstantSize = ConstU32<{ u32::MAX }>;
type MaxBuiltinSize = ConstU32<{ u32::MAX }>;
type MaxReferenceSize = ConstU32<{ u32::MAX }>;
type MaxDataSize = ConstU32<{ u32::MAX }>;
type MaxHintMapSize = ConstU32<{ u32::MAX }>;
type MaxHintSize = ConstU32<{ u32::MAX }>;
type MaxErrorMessageSize = ConstU32<{ u32::MAX }>;
type MaxInstructionLocationSize = ConstU32<{ u32::MAX }>;
type MaxIdentifiersSize = ConstU32<{ u32::MAX }>;
type MaxAccessibleScopeSize = ConstU32<{ u32::MAX }>;
// TODO: change to u128
type MaxReferenceIdsSize = ConstU32<{ u32::MAX }>;
type MaxStringLength = ConstU32<{ u32::MAX }>;
type MaxMemberLength = ConstU32<{ u32::MAX }>;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
enum MaybeRelocatableWrapper {
    RelocatableValue { segment_index: i128, offset: u128 },
    Int(Felt252Wrapper),
}

impl From<MaybeRelocatable> for MaybeRelocatableWrapper {
    fn from(value: MaybeRelocatable) -> Self {
        match value {
            MaybeRelocatable::Int(val) => Self::Int(val.into()),
            MaybeRelocatable::RelocatableValue(v) => {
                Self::RelocatableValue { segment_index: v.segment_index as i128, offset: v.offset as u128 }
            }
        }
    }
}
impl From<MaybeRelocatableWrapper> for MaybeRelocatable {
    fn from(value: MaybeRelocatableWrapper) -> Self {
        match value {
            MaybeRelocatableWrapper::Int(val) => Self::Int(val.into()),
            MaybeRelocatableWrapper::RelocatableValue { segment_index, offset } => {
                Self::RelocatableValue(Relocatable { segment_index: segment_index as isize, offset: offset as usize })
            }
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
    PartialOrd,
    Ord,
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
struct StringWrapper(BoundedVec<u8, MaxStringLength>);

impl From<String> for StringWrapper {
    /// WARNING This function can panic if the string is over 2**32-1 bytes but in our case it can
    /// never happen as the cairo compiler cannot deal with such big strings.
    fn from(value: String) -> Self {
        Self(BoundedVec::try_from(value.as_bytes().to_vec()).unwrap())
    }
}
impl From<StringWrapper> for String {
    fn from(value: StringWrapper) -> Self {
        // The cairo compiler only deals with utf-8 names so this should never panic.
        unsafe { String::from_utf8_unchecked(value.0.to_vec()) }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// This struct wraps the [Program] type from the cairo vm.
pub struct ProgramWrapper {
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "deserialize_bounded_btreemap", serialize_with = "serialize_bounded_btreemap",)
    )]
    constants: BoundedBTreeMap<StringWrapper, Felt252Wrapper, MaxConstantSize>,
    shared_program_data: SharedProgramDataWrapper,
    reference_manager: ReferenceManagerWrapper,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [ReferenceManager] using substrate compatible types.
struct ReferenceManagerWrapper {
    references: BoundedVec<ReferenceWrapper, MaxReferenceSize>,
}

impl TryFrom<ReferenceManager> for ReferenceManagerWrapper {
    type Error = VecConversionError;
    fn try_from(value: ReferenceManager) -> Result<Self, Self::Error> {
        Ok(Self { references: VecWrapper(value.references).try_into()? })
    }
}

impl From<ReferenceManagerWrapper> for ReferenceManager {
    fn from(value: ReferenceManagerWrapper) -> Self {
        Self { references: VecWrapper::from(value.references).0 }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Error type when converting a [Program] to [ProgramWrapper] and the other way around.
pub enum ProgramConversionError {
    /// Failed to convert a [HashMap] into a [BoundedBTreeMap]
    HashConversion(HashMapConversionError),
    /// Failed to convert a [Vec] into a [BoundedVec]
    VecConversion(VecConversionError),
    /// Failed to create a [Program]
    PogramCreationError,
}

impl From<HashMapConversionError> for ProgramConversionError {
    fn from(error: HashMapConversionError) -> Self {
        ProgramConversionError::HashConversion(error)
    }
}

impl From<VecConversionError> for ProgramConversionError {
    fn from(error: VecConversionError) -> Self {
        ProgramConversionError::VecConversion(error)
    }
}

impl TryFrom<Program> for ProgramWrapper {
    type Error = ProgramConversionError;

    fn try_from(value: Program) -> Result<Self, Self::Error> {
        let constants = HashMapWrapper(value.constants().clone()).try_into()?;
        let reference_manager = value.reference_manager().clone().try_into()?;

        let hints = BoundedBTreeMap::try_from(
            value
                .hints()
                .iter()
                .map(|(k, v)| VecWrapper(v.clone()).try_into().map(|v| (*k as u128, v)))
                .collect::<Result<BTreeMap<u128, BoundedVec<HintParamsWrapper, MaxHintMapSize>>, VecConversionError>>(
                )?,
        )
        .map_err(|_| ProgramConversionError::HashConversion(HashMapConversionError))?;

        let shared_program_data = SharedProgramDataWrapper {
            builtins: VecWrapper(value.builtins().clone()).try_into()?,
            data: VecWrapper(value.data().clone()).try_into()?,
            hints,
            main: value.main().map(|m| m as u128),
            start: value.start().map(|m| m as u128),
            end: value.end().map(|m| m as u128),
            error_message_attributes: VecWrapper(value.error_message_attributes().clone()).try_into()?,
            instruction_locations: match value.instruction_locations().clone() {
                Some(il) => Some(HashMapWrapper(il).try_into()?),
                None => None,
            },

            identifiers: HashMapWrapper(value.identifiers().clone()).try_into()?,
        };

        Ok(Self { constants, shared_program_data, reference_manager })
    }
}

impl TryFrom<ProgramWrapper> for Program {
    type Error = ProgramConversionError;

    fn try_from(value: ProgramWrapper) -> Result<Self, Self::Error> {
        let builtins: VecWrapper<BuiltinName> = value.shared_program_data.builtins.into();
        let data: VecWrapper<MaybeRelocatable> = value.shared_program_data.data.into();

        let hints: HashMap<usize, Vec<HintParams>> = value
            .shared_program_data
            .hints
            .into_iter()
            .map(|(k, v)| {
                // Ok to unwrap because accessible scope won't ever be too long of a string
                let v: VecWrapper<HintParams> =
                    VecWrapper(v.into_inner().iter().map(|elt| elt.clone().try_into().unwrap()).collect());
                (k as usize, v.0)
            })
            .collect::<HashMap<usize, Vec<HintParams>>>();
        Program::new(
            builtins.0,
            data.0,
            value.shared_program_data.main.map(|m| m as usize),
            hints,
            value.reference_manager.into(),
            HashMapWrapper::try_from(value.shared_program_data.identifiers)?.0,
            VecWrapper::from(value.shared_program_data.error_message_attributes).0,
            match value.shared_program_data.instruction_locations {
                Some(il) => Some(HashMapWrapper::try_from(il)?.0),
                None => None,
            },
        )
        .map_err(|_| ProgramConversionError::PogramCreationError)
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
    Copy,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [Felt252] using [U256] (substrate compatible type).
pub struct Felt252Wrapper(pub U256);

impl From<Felt252> for Felt252Wrapper {
    fn from(value: Felt252) -> Self {
        Self(U256::from_big_endian(&value.to_be_bytes()))
    }
}
impl From<Felt252Wrapper> for Felt252 {
    fn from(value: Felt252Wrapper) -> Self {
        let mut buff: [u8; 32] = [0u8; 32];
        value.0.to_big_endian(&mut buff);
        Felt252::from_bytes_be(&buff)
    }
}
impl From<H256> for Felt252Wrapper {
    fn from(value: H256) -> Self {
        Felt252Wrapper(U256::from_big_endian(value.as_bytes()))
    }
}
impl From<Felt252Wrapper> for H256 {
    fn from(value: Felt252Wrapper) -> Self {
        let mut buff: [u8; 32] = [0u8; 32];
        value.0.to_big_endian(&mut buff);
        H256::from_slice(&buff)
    }
}
impl From<Felt252Wrapper> for FieldElement {
    fn from(value: Felt252Wrapper) -> Self {
        let mut buff: [u8; 32] = [0u8; 32];
        value.0.to_big_endian(&mut buff);
        FieldElement::from_byte_slice_be(&buff).unwrap()
    }
}
impl From<FieldElement> for Felt252Wrapper {
    fn from(value: FieldElement) -> Self {
        value.to_bytes_be().into()
    }
}
impl From<Felt252Wrapper> for [u8; 32] {
    fn from(value: Felt252Wrapper) -> Self {
        let mut buff: [u8; 32] = [0u8; 32];
        value.0.to_big_endian(&mut buff);
        buff
    }
}
impl From<&[u8]> for Felt252Wrapper {
    fn from(value: &[u8]) -> Self {
        Felt252Wrapper(U256::from_big_endian(value))
    }
}
impl From<[u8; 32]> for Felt252Wrapper {
    fn from(value: [u8; 32]) -> Self {
        Felt252Wrapper(U256::from_big_endian(&value))
    }
}

impl Felt252Wrapper {
    /// Returns the zero value.
    pub fn zero() -> Self {
        Self(U256::zero())
    }

    /// Returns the one value.
    pub fn one() -> Self {
        Self(U256::one())
    }

    /// Return the max 128 bits value.
    pub fn max_u128() -> Self {
        Self(U256::from(u128::MAX))
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Error type when converting a [Vec] to [BoundedVec] and the other way around.
pub struct VecConversionError;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [Vec] using [BoundedVec] (substrate compatible type). We wrap this in order to
/// be able to easily convert a [Vec<T>] into a [BoundedVec<T, Size>] by implementing the [From] and
/// [TryFrom] traits.
struct VecWrapper<T>(Vec<T>);

impl<T> Default for VecWrapper<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<O, D, S> TryFrom<VecWrapper<O>> for BoundedVec<D, S>
where
    S: Get<u32>,
    D: TryFrom<O>,
    O: Clone,
{
    type Error = VecConversionError;
    fn try_from(value: VecWrapper<O>) -> Result<Self, Self::Error> {
        let bv = value
            .0
            .into_iter()
            .map(|elt| elt.try_into().map_err(|_| VecConversionError))
            .collect::<Result<Vec<D>, Self::Error>>()?;
        BoundedVec::try_from(bv).map_err(|_| VecConversionError)
    }
}
impl<O, D, S> From<BoundedVec<O, S>> for VecWrapper<D>
where
    S: Get<u32>,
    D: From<O>,
    O: Clone,
{
    fn from(value: BoundedVec<O, S>) -> Self {
        VecWrapper::<D>(value.into_inner().iter().map(|elt| D::from(elt.clone())).collect())
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Error type when converting a [HashMap] to [BoundedBTreeMap] and the other way around.
pub struct HashMapConversionError;
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [HashMap] using [BoundedBTreeMap] (substrate compatible type). We wrap this in
/// order to be able to easily convert a [HashMap<K, V>] into a [BoundedBTreeMap<K, V, Size>] by
/// implementing the [From] and [TryFrom] traits.
struct HashMapWrapper<K, V>(HashMap<K, V>)
where
    K: Eq + Hash + Ord;

impl<KEY, VALUE, K, V, S> TryFrom<HashMapWrapper<KEY, VALUE>> for BoundedBTreeMap<K, V, S>
where
    K: TryFrom<KEY> + Clone + Ord,
    V: TryFrom<VALUE> + Clone,
    <V as TryFrom<VALUE>>::Error: Debug,
    S: Get<u32>,
    KEY: Eq + Hash + Ord,
{
    type Error = HashMapConversionError;
    fn try_from(value: HashMapWrapper<KEY, VALUE>) -> Result<Self, Self::Error> {
        let btree = value
            .0
            .into_iter()
            .map(|(key, val)| match (key.try_into(), val.try_into()) {
                (Ok(key), Ok(val)) => Ok((key, val)),
                _ => Err(HashMapConversionError),
            })
            .collect::<Result<BTreeMap<K, V>, HashMapConversionError>>()?;
        BoundedBTreeMap::try_from(btree).map_err(|_| HashMapConversionError)
    }
}

impl<KEY, VALUE, K, V, S> TryFrom<BoundedBTreeMap<K, V, S>> for HashMapWrapper<KEY, VALUE>
where
    KEY: Eq + Hash + TryFrom<K> + Ord,
    VALUE: TryFrom<V>,
    <VALUE as TryFrom<V>>::Error: Debug,
{
    type Error = HashMapConversionError;

    fn try_from(value: BoundedBTreeMap<K, V, S>) -> Result<Self, Self::Error> {
        value
            .into_iter()
            .map(|(key, val)| match (key.try_into(), val.try_into()) {
                (Ok(key), Ok(val)) => Ok((key, val)),
                _ => Err(HashMapConversionError),
            })
            .collect::<Result<HashMap<KEY, VALUE>, HashMapConversionError>>()
            .map(HashMapWrapper)
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [SharedProgramData] using substrate compatible types.
struct SharedProgramDataWrapper {
    builtins: BoundedVec<BuiltinNameWrapper, MaxBuiltinSize>,
    data: BoundedVec<MaybeRelocatableWrapper, MaxDataSize>,
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "deserialize_bounded_btreemap", serialize_with = "serialize_bounded_btreemap")
    )]
    hints: BoundedBTreeMap<u128, BoundedVec<HintParamsWrapper, MaxHintSize>, MaxHintMapSize>,
    main: Option<u128>,
    // start and end labels will only be used in proof-mode
    start: Option<u128>,
    end: Option<u128>,
    error_message_attributes: BoundedVec<AttributeWrapper, MaxErrorMessageSize>,
    #[cfg_attr(
        feature = "std",
        serde(
            deserialize_with = "deserialize_option_bounded_btreemap",
            serialize_with = "serialize_option_bounded_btreemap"
        )
    )]
    instruction_locations: Option<BoundedBTreeMap<u128, InstructionLocationWrapper, MaxInstructionLocationSize>>,
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "deserialize_bounded_btreemap", serialize_with = "serialize_bounded_btreemap")
    )]
    identifiers: BoundedBTreeMap<StringWrapper, IdentifierWrapper, MaxIdentifiersSize>,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
/// Wrapper type from [BuiltinName] using (substrate compatible type).
enum BuiltinNameWrapper {
    /// Output builtin.
    output,
    /// Range check builtin.
    range_check,
    /// Pedersen builtin.
    pedersen,
    /// Ecdsa builtin.
    ecdsa,
    /// Keccak builtin.
    keccak,
    /// Bitwise builtin.
    bitwise,
    /// Ec op builtin.
    ec_op,
    /// Poseidon builtin.
    poseidon,
}

impl From<BuiltinName> for BuiltinNameWrapper {
    fn from(value: BuiltinName) -> Self {
        match value {
            BuiltinName::output => Self::output,
            BuiltinName::range_check => Self::range_check,
            BuiltinName::pedersen => Self::pedersen,
            BuiltinName::ecdsa => Self::ecdsa,
            BuiltinName::keccak => Self::keccak,
            BuiltinName::bitwise => Self::bitwise,
            BuiltinName::ec_op => Self::ec_op,
            BuiltinName::poseidon => Self::poseidon,
        }
    }
}

impl From<BuiltinNameWrapper> for BuiltinName {
    fn from(value: BuiltinNameWrapper) -> Self {
        match value {
            BuiltinNameWrapper::output => Self::output,
            BuiltinNameWrapper::range_check => Self::range_check,
            BuiltinNameWrapper::pedersen => Self::pedersen,
            BuiltinNameWrapper::ecdsa => Self::ecdsa,
            BuiltinNameWrapper::keccak => Self::keccak,
            BuiltinNameWrapper::bitwise => Self::bitwise,
            BuiltinNameWrapper::ec_op => Self::ec_op,
            BuiltinNameWrapper::poseidon => Self::poseidon,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [HintParams] using (substrate compatible type).
struct HintParamsWrapper {
    code: StringWrapper,
    accessible_scopes: BoundedVec<StringWrapper, MaxAccessibleScopeSize>,
    flow_tracking_data: FlowTrackingDataWrapper,
}

impl TryFrom<HintParams> for HintParamsWrapper {
    type Error = VecConversionError;
    fn try_from(value: HintParams) -> Result<Self, Self::Error> {
        Ok(Self {
            code: value.code.into(),
            accessible_scopes: VecWrapper::<String>(value.accessible_scopes).try_into()?,
            flow_tracking_data: value.flow_tracking_data.into(),
        })
    }
}
impl From<HintParamsWrapper> for HintParams {
    fn from(value: HintParamsWrapper) -> Self {
        Self {
            code: value.code.into(),
            accessible_scopes: value.accessible_scopes.into_inner().iter().map(|scope| scope.clone().into()).collect(),
            flow_tracking_data: value.flow_tracking_data.into(),
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [FlowTrackingData] using (substrate compatible type).
struct FlowTrackingDataWrapper {
    ap_tracking: ApTrackingWrapper,
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "deserialize_bounded_btreemap", serialize_with = "serialize_bounded_btreemap")
    )]
    reference_ids: BoundedBTreeMap<StringWrapper, u128, MaxReferenceIdsSize>,
}

impl From<FlowTrackingData> for FlowTrackingDataWrapper {
    fn from(value: FlowTrackingData) -> Self {
        // When the map size will be u128 it will never overflow because references can go only up to u128
        Self {
            ap_tracking: value.ap_tracking.into(),
            reference_ids: HashMapWrapper(value.reference_ids).try_into().unwrap(),
        }
    }
}
impl From<FlowTrackingDataWrapper> for FlowTrackingData {
    fn from(value: FlowTrackingDataWrapper) -> Self {
        // When the map size will be u128 it will never overflow because references can go only up to u128
        Self {
            ap_tracking: value.ap_tracking.into(),
            reference_ids: HashMapWrapper::<String, usize>::try_from(value.reference_ids).unwrap().0,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [ApTracking] using (substrate compatible type).
pub struct ApTrackingWrapper {
    /// Ap group.
    pub group: u128,
    /// Ap offset.
    pub offset: u128,
}

impl From<ApTracking> for ApTrackingWrapper {
    fn from(value: ApTracking) -> Self {
        Self { group: value.group as u128, offset: value.offset as u128 }
    }
}
impl From<ApTrackingWrapper> for ApTracking {
    fn from(value: ApTrackingWrapper) -> Self {
        Self { group: value.group as usize, offset: value.offset as usize }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [InstructionLocation] using (substrate compatible type).
pub struct InstructionLocationWrapper {
    inst: LocationWrapper,
    hints: BoundedVec<HintLocationWrapper, MaxHintSize>,
}

impl From<InstructionLocation> for InstructionLocationWrapper {
    fn from(value: InstructionLocation) -> Self {
        Self {
            inst: value.inst.into(),
            hints: value
                .hints
                .iter()
                .map(|hint| hint.clone().into())
                .collect::<Vec<HintLocationWrapper>>()
                .try_into()
                .unwrap(),
        }
    }
}

impl From<InstructionLocationWrapper> for InstructionLocation {
    fn from(value: InstructionLocationWrapper) -> Self {
        Self {
            inst: value.inst.into(),
            hints: value.hints.into_inner().iter().map(|hint| hint.clone().into()).collect(),
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [Location] using (substrate compatible type).
pub struct LocationWrapper {
    end_line: u32,
    end_col: u32,
    input_file: StringWrapper,
    parent_location: Option<(Box<LocationWrapper>, StringWrapper)>,
    start_line: u32,
    start_col: u32,
}

impl From<Location> for LocationWrapper {
    fn from(value: Location) -> Self {
        let InputFile { filename } = value.input_file;
        let parent_loc = value.parent_location.map(|(loc, name)| (Box::from(LocationWrapper::from(*loc)), name.into()));
        Self {
            end_line: value.end_line,
            end_col: value.end_col,
            input_file: filename.into(),
            parent_location: parent_loc,
            start_line: value.start_line,
            start_col: value.start_col,
        }
    }
}

impl From<LocationWrapper> for Location {
    fn from(value: LocationWrapper) -> Self {
        let parent_loc = value.parent_location.map(|(loc, name)| (Box::from(Location::from(*loc)), name.into()));
        Self {
            end_line: value.end_line,
            end_col: value.end_col,
            input_file: InputFile { filename: value.input_file.into() },
            parent_location: parent_loc,
            start_line: value.start_line,
            start_col: value.start_col,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [HintLocation] using (substrate compatible type).
pub struct HintLocationWrapper {
    location: LocationWrapper,
    n_prefix_newlines: u32,
}

impl From<HintLocation> for HintLocationWrapper {
    fn from(value: HintLocation) -> Self {
        Self { location: value.location.into(), n_prefix_newlines: value.n_prefix_newlines }
    }
}
impl From<HintLocationWrapper> for HintLocation {
    fn from(value: HintLocationWrapper) -> Self {
        Self { location: value.location.into(), n_prefix_newlines: value.n_prefix_newlines }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [Attribute] using (substrate compatible type).
struct AttributeWrapper {
    name: StringWrapper,
    start_pc: u128,
    end_pc: u128,
    value: StringWrapper,
    flow_tracking_data: Option<FlowTrackingDataWrapper>,
}

impl From<Attribute> for AttributeWrapper {
    fn from(value: Attribute) -> Self {
        Self {
            name: value.name.into(),
            start_pc: value.start_pc as u128,
            end_pc: value.end_pc as u128,
            value: value.value.into(),
            flow_tracking_data: value.flow_tracking_data.map(|flow| flow.into()),
        }
    }
}

impl From<AttributeWrapper> for Attribute {
    fn from(value: AttributeWrapper) -> Self {
        Self {
            name: value.name.into(),
            start_pc: value.start_pc as usize,
            end_pc: value.end_pc as usize,
            value: value.value.into(),
            // Only way it panics is if u128 to usize fails which is safe so we can unwrap
            flow_tracking_data: value.flow_tracking_data.map(|flow| flow.try_into().unwrap()),
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [Identifier] using (substrate compatible type).
struct IdentifierWrapper {
    pc: Option<u128>,
    #[cfg_attr(feature = "std", serde(rename(deserialize = "type")))]
    type_: Option<StringWrapper>,
    value: Option<Felt252Wrapper>,

    full_name: Option<StringWrapper>,
    #[cfg_attr(
        feature = "std",
        serde(
            deserialize_with = "deserialize_option_bounded_btreemap",
            serialize_with = "serialize_option_bounded_btreemap"
        )
    )]
    members: Option<BoundedBTreeMap<StringWrapper, MemberWrapper, MaxMemberLength>>,
    cairo_type: Option<StringWrapper>,
}
impl From<Identifier> for IdentifierWrapper {
    fn from(value: Identifier) -> Self {
        Self {
            pc: value.pc.map(|v| v as u128),
            type_: value.type_.map(|v| v.into()),
            value: value.value.map(|v| v.into()),
            full_name: value.full_name.map(|v| v.into()),
            // Nothing should have more than 2**32-1 members so it shouldn't panic.
            members: value.members.map(|v| HashMapWrapper(v).try_into().unwrap()),
            cairo_type: value.cairo_type.map(|v| v.into()),
        }
    }
}
impl From<IdentifierWrapper> for Identifier {
    fn from(value: IdentifierWrapper) -> Self {
        Self {
            pc: value.pc.map(|v| v as usize),
            type_: value.type_.map(|v| v.into()),
            value: value.value.map(|v| v.into()),
            full_name: value.full_name.map(|v| v.into()),
            // Nothing should have more than 2**32-1 members so it shouldn't panic.
            members: value.members.map(|v| {
                let hash_map: HashMapWrapper<String, Member> = v.try_into().unwrap();
                hash_map.0
            }),
            cairo_type: value.cairo_type.map(|v| v.into()),
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [Member] using (substrate compatible type).
struct MemberWrapper {
    cairo_type: StringWrapper,
    offset: u128,
}

impl From<Member> for MemberWrapper {
    fn from(value: Member) -> Self {
        Self { cairo_type: value.cairo_type.into(), offset: value.offset as u128 }
    }
}
impl From<MemberWrapper> for Member {
    fn from(value: MemberWrapper) -> Self {
        Self { cairo_type: value.cairo_type.into(), offset: value.offset as usize }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [Reference] using (substrate compatible type).
struct ReferenceWrapper {
    /// Ap tracking data.
    ap_tracking_data: ApTrackingWrapper,
    /// Program counter.
    pc: Option<u128>,
    /// Address of the reference.
    value_address: ValueAddressWrapper,
}

impl From<Reference> for ReferenceWrapper {
    fn from(value: Reference) -> Self {
        Self {
            ap_tracking_data: value.ap_tracking_data.into(),
            pc: value.pc.map(|v| v as u128),
            value_address: value.value_address.into(),
        }
    }
}
impl From<ReferenceWrapper> for Reference {
    fn from(value: ReferenceWrapper) -> Self {
        Self {
            ap_tracking_data: value.ap_tracking_data.into(),
            pc: value.pc.map(|v| v as usize),
            value_address: value.value_address.into(),
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [OffsetValue] using (substrate compatible type).
pub enum OffsetValueWrapper {
    /// Immediate.
    Immediate(Felt252Wrapper),
    /// Value.
    Value(i32),
    /// Reference.
    Reference(RegisterWrapper, i32, bool),
}

impl From<OffsetValue> for OffsetValueWrapper {
    fn from(value: OffsetValue) -> Self {
        match value {
            OffsetValue::Immediate(val) => Self::Immediate(val.into()),
            OffsetValue::Value(val) => Self::Value(val),
            OffsetValue::Reference(register, val, b) => Self::Reference(register.into(), val, b),
        }
    }
}
impl From<OffsetValueWrapper> for OffsetValue {
    fn from(value: OffsetValueWrapper) -> Self {
        match value {
            OffsetValueWrapper::Immediate(val) => Self::Immediate(val.into()),
            OffsetValueWrapper::Value(val) => Self::Value(val),
            OffsetValueWrapper::Reference(register, val, b) => Self::Reference(register.into(), val, b),
        }
    }
}

impl Default for OffsetValueWrapper {
    fn default() -> Self {
        Self::Value(0)
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    Default,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [ValueAddress] using (substrate compatible type).
pub struct ValueAddressWrapper {
    /// First offset.
    pub offset1: OffsetValueWrapper,
    /// Second offset.
    pub offset2: OffsetValueWrapper,
    /// Dereference.
    pub dereference: bool,
    value_type: StringWrapper,
}

impl From<ValueAddress> for ValueAddressWrapper {
    fn from(value: ValueAddress) -> Self {
        Self {
            offset1: value.offset1.into(),
            offset2: value.offset2.into(),
            dereference: value.dereference,
            value_type: value.value_type.into(),
        }
    }
}

impl From<ValueAddressWrapper> for ValueAddress {
    fn from(value: ValueAddressWrapper) -> Self {
        Self {
            offset1: value.offset1.into(),
            offset2: value.offset2.into(),
            dereference: value.dereference,
            value_type: value.value_type.into(),
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [Register] using (substrate compatible type).
pub enum RegisterWrapper {
    /// AP.
    AP,
    /// FP.
    FP,
}

impl From<Register> for RegisterWrapper {
    fn from(value: Register) -> Self {
        match value {
            Register::AP => Self::AP,
            Register::FP => Self::FP,
        }
    }
}
impl From<RegisterWrapper> for Register {
    fn from(value: RegisterWrapper) -> Self {
        match value {
            RegisterWrapper::AP => Self::AP,
            RegisterWrapper::FP => Self::FP,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Wrapper type from [Instruction] using (substrate compatible type).
struct InstructionWrapper {
    off0: u128,
    off1: u128,
    off2: u128,
    imm: Option<Felt252Wrapper>,
    dst_register: RegisterWrapper,
    op0_register: RegisterWrapper,
    op1_addr: Op1AddrWrapper,
    res: ResWrapper,
    pc_update: PcUpdateWrapper,
    ap_update: ApUpdateWrapper,
    fp_update: FpUpdateWrapper,
    opcode: OpcodeWrapper,
}

impl From<Instruction> for InstructionWrapper {
    fn from(value: Instruction) -> Self {
        Self {
            off0: value.off0 as u128,
            off1: value.off1 as u128,
            off2: value.off2 as u128,
            imm: value.imm.map(Felt252Wrapper::from),
            dst_register: value.dst_register.into(),
            op0_register: value.op0_register.into(),
            op1_addr: value.op1_addr.into(),
            res: value.res.into(),
            pc_update: value.pc_update.into(),
            ap_update: value.ap_update.into(),
            fp_update: value.fp_update.into(),
            opcode: value.opcode.into(),
        }
    }
}

impl From<InstructionWrapper> for Instruction {
    fn from(value: InstructionWrapper) -> Self {
        Self {
            off0: value.off0 as isize,
            off1: value.off1 as isize,
            off2: value.off2 as isize,
            imm: value.imm.map(Felt252::from),
            dst_register: value.dst_register.into(),
            op0_register: value.op0_register.into(),
            op1_addr: value.op1_addr.into(),
            res: value.res.into(),
            pc_update: value.pc_update.into(),
            ap_update: value.ap_update.into(),
            fp_update: value.fp_update.into(),
            opcode: value.opcode.into(),
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
enum Op1AddrWrapper {
    Imm,
    AP,
    FP,
    Op0,
}

impl From<Op1Addr> for Op1AddrWrapper {
    fn from(value: Op1Addr) -> Self {
        match value {
            Op1Addr::Imm => Self::Imm,
            Op1Addr::AP => Self::AP,
            Op1Addr::FP => Self::FP,
            Op1Addr::Op0 => Self::Op0,
        }
    }
}

impl From<Op1AddrWrapper> for Op1Addr {
    fn from(value: Op1AddrWrapper) -> Self {
        match value {
            Op1AddrWrapper::Imm => Self::Imm,
            Op1AddrWrapper::AP => Self::AP,
            Op1AddrWrapper::FP => Self::FP,
            Op1AddrWrapper::Op0 => Self::Op0,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
enum ResWrapper {
    Op1,
    Add,
    Mul,
    Unconstrained,
}

impl From<Res> for ResWrapper {
    fn from(value: Res) -> Self {
        match value {
            Res::Op1 => Self::Op1,
            Res::Add => Self::Add,
            Res::Mul => Self::Mul,
            Res::Unconstrained => Self::Unconstrained,
        }
    }
}

impl From<ResWrapper> for Res {
    fn from(value: ResWrapper) -> Self {
        match value {
            ResWrapper::Op1 => Self::Op1,
            ResWrapper::Add => Self::Add,
            ResWrapper::Mul => Self::Mul,
            ResWrapper::Unconstrained => Self::Unconstrained,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
enum PcUpdateWrapper {
    Regular,
    Jump,
    JumpRel,
    Jnz,
}

impl From<PcUpdate> for PcUpdateWrapper {
    fn from(value: PcUpdate) -> Self {
        match value {
            PcUpdate::Regular => Self::Regular,
            PcUpdate::Jump => Self::Jump,
            PcUpdate::JumpRel => Self::JumpRel,
            PcUpdate::Jnz => Self::Jnz,
        }
    }
}
impl From<PcUpdateWrapper> for PcUpdate {
    fn from(value: PcUpdateWrapper) -> Self {
        match value {
            PcUpdateWrapper::Regular => Self::Regular,
            PcUpdateWrapper::Jump => Self::Jump,
            PcUpdateWrapper::JumpRel => Self::JumpRel,
            PcUpdateWrapper::Jnz => Self::Jnz,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
enum ApUpdateWrapper {
    Regular,
    Add,
    Add1,
    Add2,
}

impl From<ApUpdate> for ApUpdateWrapper {
    fn from(value: ApUpdate) -> Self {
        match value {
            ApUpdate::Regular => Self::Regular,
            ApUpdate::Add => Self::Add,
            ApUpdate::Add1 => Self::Add1,
            ApUpdate::Add2 => Self::Add2,
        }
    }
}

impl From<ApUpdateWrapper> for ApUpdate {
    fn from(value: ApUpdateWrapper) -> Self {
        match value {
            ApUpdateWrapper::Regular => Self::Regular,
            ApUpdateWrapper::Add => Self::Add,
            ApUpdateWrapper::Add1 => Self::Add1,
            ApUpdateWrapper::Add2 => Self::Add2,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
enum FpUpdateWrapper {
    Regular,
    APPlus2,
    Dst,
}

impl From<FpUpdate> for FpUpdateWrapper {
    fn from(value: FpUpdate) -> Self {
        match value {
            FpUpdate::Regular => Self::Regular,
            FpUpdate::APPlus2 => Self::APPlus2,
            FpUpdate::Dst => Self::Dst,
        }
    }
}

impl From<FpUpdateWrapper> for FpUpdate {
    fn from(value: FpUpdateWrapper) -> Self {
        match value {
            FpUpdateWrapper::Regular => Self::Regular,
            FpUpdateWrapper::APPlus2 => Self::APPlus2,
            FpUpdateWrapper::Dst => Self::Dst,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
enum OpcodeWrapper {
    NOp,
    AssertEq,
    Call,
    Ret,
}

impl From<Opcode> for OpcodeWrapper {
    fn from(value: Opcode) -> Self {
        match value {
            Opcode::AssertEq => Self::AssertEq,
            Opcode::Call => Self::Call,
            Opcode::NOp => Self::NOp,
            Opcode::Ret => Self::Ret,
        }
    }
}
impl From<OpcodeWrapper> for Opcode {
    fn from(value: OpcodeWrapper) -> Self {
        match value {
            OpcodeWrapper::AssertEq => Self::AssertEq,
            OpcodeWrapper::Call => Self::Call,
            OpcodeWrapper::NOp => Self::NOp,
            OpcodeWrapper::Ret => Self::Ret,
        }
    }
}
