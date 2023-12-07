use serde::Serializer;
use serde_with::SerializeAs;

use crate::Felt252Wrapper;

pub struct UfeHex;

impl SerializeAs<Felt252Wrapper> for UfeHex {
    fn serialize_as<S>(value: &Felt252Wrapper, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        starknet_core::serde::unsigned_field_element::UfeHex::serialize_as::<S>(&value.0, serializer)
    }
}
