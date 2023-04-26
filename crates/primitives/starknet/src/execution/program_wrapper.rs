use alloc::boxed::Box;

use derive_more::Constructor;
use frame_support::{BoundedBTreeMap, BoundedVec};
use sp_core::{ConstU32, U256};

#[cfg(feature = "std")]
use super::{
    deserialize_bounded_btreemap, deserialize_option_bounded_btreemap, serialize_bounded_btreemap,
    serialize_option_bounded_btreemap,
};

type MaxConstantSize = ConstU32<{ u32::MAX }>;
type MaxBuiltinSize = ConstU32<{ u32::MAX }>;
type MaxDataSize = ConstU32<{ u32::MAX }>;
type MaxHintMapSize = ConstU32<{ u32::MAX }>;
type MaxHintSize = ConstU32<{ u32::MAX }>;
type MaxErrorMessageSize = ConstU32<{ u32::MAX }>;
type MaxInstructionLocationSize = ConstU32<{ u32::MAX }>;
type MaxIdentifiersSize = ConstU32<{ u32::MAX }>;
type MaxAccessibleScopeSize = ConstU32<{ u32::MAX }>;
type MaxReferenceIdsSize = ConstU32<{ u32::MAX }>;
type MaxStringLength = ConstU32<{ u32::MAX }>;
type MaxMemberLength = ConstU32<{ u32::MAX }>;
type String = BoundedVec<u8, MaxStringLength>;
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
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ProgramWrapper {
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "deserialize_bounded_btreemap", serialize_with = "serialize_bounded_btreemap")
    )]
    constants: BoundedBTreeMap<String, U256, MaxConstantSize>,
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
    identifiers: BoundedBTreeMap<String, IdentifierWrapper, MaxIdentifiersSize>,
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
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct HintParamsWrapper {
    code: String,
    accessible_scopes: BoundedVec<String, MaxAccessibleScopeSize>,
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
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct FlowTrackingDataWrapper {
    ap_tracking: ApTrackingWrapper,
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "deserialize_bounded_btreemap", serialize_with = "serialize_bounded_btreemap")
    )]
    reference_ids: BoundedBTreeMap<String, u128, MaxReferenceIdsSize>,
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
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ApTrackingWrapper {
    group: u128,
    offset: u128,
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
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct LocationWrapper {
    end_line: u32,
    end_col: u32,
    input_file: String,
    parent_location: Option<(Box<LocationWrapper>, String)>,
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
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeWrapper {
    name: String,
    start_pc: u128,
    end_pc: u128,
    value: String,
    flow_tracking_data: Option<FlowTrackingDataWrapper>,
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
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierWrapper {
    pc: Option<u128>,
    #[cfg_attr(feature = "std", serde(rename(deserialize = "type")))]
    type_: Option<String>,
    value: Option<U256>,

    full_name: Option<String>,
    #[cfg_attr(
        feature = "std",
        serde(
            deserialize_with = "deserialize_option_bounded_btreemap",
            serialize_with = "serialize_option_bounded_btreemap"
        )
    )]
    members: Option<BoundedBTreeMap<String, MemberWrapper, MaxMemberLength>>,
    cairo_type: Option<String>,
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
    Constructor,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct MemberWrapper {
    cairo_type: String,
    offset: u128,
}
