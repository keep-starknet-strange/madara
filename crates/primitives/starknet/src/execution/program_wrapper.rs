use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::cmp::Eq;
use core::fmt::Debug;
use core::hash::Hash;

use cairo_vm::felt::Felt252;
use cairo_vm::serde::deserialize_program::{
    ApTracking, Attribute, FlowTrackingData, Identifier, Member, OffsetValue, Reference, ValueAddress,
};
use cairo_vm::types::instruction::Register;
use cairo_vm::types::program::Program;
use derive_more::Constructor;
use frame_support::{BoundedBTreeMap, BoundedVec};
use sp_core::{ConstU32, Get, U256};
use starknet_api::stdlib::collections::HashMap;

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
type MaybeRelocatableWrapper = U256;
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
struct StringConversionError;

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
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ProgramWrapper {
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "deserialize_bounded_btreemap", serialize_with = "serialize_bounded_btreemap")
    )]
    constants: BoundedBTreeMap<StringWrapper, Felt252Wrapper, MaxConstantSize>,
    shared_program_data: SharedProgramDataWrapper,
    reference_manager: BoundedVec<ReferenceWrapper, MaxReferenceSize>,
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
struct Felt252Wrapper(pub U256);

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
struct HashMapConversionError;
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
struct HashMapWrapper<K, V>(HashMap<K, V>)
where
    K: Eq + Hash;

impl<KEY, VALUE, K, V, S> TryFrom<HashMapWrapper<KEY, VALUE>> for BoundedBTreeMap<K, V, S>
where
    K: From<KEY> + Clone + Ord,
    V: TryFrom<VALUE> + Clone,
    <V as TryFrom<VALUE>>::Error: Debug,
    S: Get<u32>,
    KEY: Eq + Hash,
{
    type Error = HashMapConversionError;
    fn try_from(value: HashMapWrapper<KEY, VALUE>) -> Result<Self, Self::Error> {
        let btree = value
            .0
            .iter()
            .map(|(&key, &val)| val.try_into().map(|v| (key.into(), v)))
            .collect::<Result<BTreeMap<K, V>, <V as TryFrom<VALUE>>::Error>>()
            .map_err(|_| HashMapConversionError)?;
        BoundedBTreeMap::try_from(btree).map_err(|_| HashMapConversionError)
    }
}

impl<KEY, VALUE, K, V, S> From<BoundedBTreeMap<K, V, S>> for HashMapWrapper<KEY, VALUE>
where
    KEY: Eq + Hash + From<K>,
    VALUE: From<V>,
{
    fn from(value: BoundedBTreeMap<K, V, S>) -> Self {
        let hash_map = value.into_iter().map(|(key, val)| (key.into(), val.into())).collect::<HashMap<KEY, VALUE>>();
        HashMapWrapper(hash_map)
    }
}

impl From<Program> for ProgramWrapper {
    fn from(value: Program) -> Self {
        ProgramWrapper::default()
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
pub struct SharedProgramDataWrapper {
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
pub enum BuiltinNameWrapper {
    output,
    range_check,
    pedersen,
    ecdsa,
    keccak,
    bitwise,
    ec_op,
    poseidon,
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
pub struct HintParamsWrapper {
    code: StringWrapper,
    accessible_scopes: BoundedVec<StringWrapper, MaxAccessibleScopeSize>,
    flow_tracking_data: FlowTrackingDataWrapper,
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
pub struct FlowTrackingDataWrapper {
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

// DONE
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
pub struct ApTrackingWrapper {
    pub group: u128,
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
pub struct InstructionLocationWrapper {
    inst: LocationWrapper,
    hints: BoundedVec<HintLocationWrapper, MaxHintSize>,
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
pub struct LocationWrapper {
    end_line: u32,
    end_col: u32,
    input_file: StringWrapper,
    parent_location: Option<(Box<LocationWrapper>, StringWrapper)>,
    start_line: u32,
    start_col: u32,
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
pub struct HintLocationWrapper {
    location: LocationWrapper,
    n_prefix_newlines: u32,
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
pub struct AttributeWrapper {
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

// DONE
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
pub struct IdentifierWrapper {
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
                let hash_map: HashMapWrapper<String, Member> = v.into();
                hash_map.0
            }),
            cairo_type: value.cairo_type.map(|v| v.into()),
        }
    }
}

// DONE
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
pub struct MemberWrapper {
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

// DONE

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
pub struct ReferenceWrapper {
    pub ap_tracking_data: ApTrackingWrapper,
    pub pc: Option<u128>,
    pub value_address: ValueAddressWrapper,
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

// DONE
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
pub enum OffsetValueWrapper {
    Immediate(Felt252Wrapper),
    Value(i32),
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

// DONE
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
pub struct ValueAddressWrapper {
    pub offset1: OffsetValueWrapper,
    pub offset2: OffsetValueWrapper,
    pub dereference: bool,
    pub value_type: StringWrapper,
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

// DONE
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
pub enum RegisterWrapper {
    AP,
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
