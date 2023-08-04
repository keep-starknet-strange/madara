use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use scale_codec::{Decode, Encode, EncodeAsRef};

type ActualResources = BTreeMap<String, usize>;

/// Wrapper for `actual_resources` field type in TransactionExecutionInfoWrapper
#[derive(Encode, Decode)]
pub struct ActualResourcesCodec(Vec<(String, u64)>);

impl From<ActualResourcesCodec> for ActualResources {
    fn from(value: ActualResourcesCodec) -> Self {
        value.0.into_iter().map(|(k, v)| (k, v as usize)).collect()
    }
}

impl From<&ActualResources> for ActualResourcesCodec {
    fn from(value: &ActualResources) -> Self {
        Self(value.clone().into_iter().map(|(k, v)| (k, v as u64)).collect())
    }
}

impl EncodeAsRef<'_, ActualResources> for ActualResourcesCodec {
    type RefType = ActualResourcesCodec;
}
