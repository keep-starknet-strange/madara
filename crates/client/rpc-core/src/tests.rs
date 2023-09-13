use starknet_core::types::BlockTag;

use super::*;

#[test]
fn block_id_serialization() {
    assert_eq!(serde_json::to_value(BlockId::Number(42)).unwrap(), serde_json::json!({"block_number": 42}));
    assert_eq!(
        serde_json::to_value(BlockId::Hash(FieldElement::from_hex_be("0x42").unwrap())).unwrap(),
        serde_json::json!({"block_hash": "0x42"})
    );
    assert_eq!(serde_json::to_value(BlockId::Tag(BlockTag::Latest)).unwrap(), "latest");
    assert_eq!(serde_json::to_value(BlockId::Tag(BlockTag::Pending)).unwrap(), "pending");
}

#[test]
fn block_id_deserialization() {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Payload {
        #[serde(rename = "block_id")]
        block_id: BlockId,
    }

    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": \"latest\" }").unwrap().block_id,
        BlockId::Tag(BlockTag::Latest)
    );
    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": \"pending\" }").unwrap().block_id,
        BlockId::Tag(BlockTag::Pending)
    );
    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": { \"block_hash\": \"0x42\"} }").unwrap().block_id,
        BlockId::Hash(FieldElement::from_hex_be("0x42").unwrap())
    );
    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": { \"block_number\": 42} }").unwrap().block_id,
        BlockId::Number(42)
    );
}
