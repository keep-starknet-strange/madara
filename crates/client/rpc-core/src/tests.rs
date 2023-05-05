use super::*;

#[test]
fn block_id_serialization() {
    assert_eq!(serde_json::to_value(BlockId::BlockNumber(42)).unwrap(), serde_json::json!({"block_number": 42}));
    assert_eq!(
        serde_json::to_value(BlockId::BlockHash("0x42".to_string())).unwrap(),
        serde_json::json!({"block_hash": "0x42"})
    );
    assert_eq!(serde_json::to_value(BlockId::BlockTag(BlockTag::Latest)).unwrap(), "latest");
    assert_eq!(serde_json::to_value(BlockId::BlockTag(BlockTag::Pending)).unwrap(), "pending");
}

#[test]
fn block_id_deserialization() {
    #[derive(Serialize, Deserialize)]
    struct Payload {
        #[serde(rename = "block_id")]
        block_id: BlockId,
    }

    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": \"latest\" }").unwrap().block_id,
        BlockId::BlockTag(BlockTag::Latest)
    );
    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": \"pending\" }").unwrap().block_id,
        BlockId::BlockTag(BlockTag::Pending)
    );
    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": { \"block_hash\": \"0x42\"} }").unwrap().block_id,
        BlockId::BlockHash("0x42".to_string())
    );
    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": { \"block_number\": 42} }").unwrap().block_id,
        BlockId::BlockNumber(42)
    );
}
