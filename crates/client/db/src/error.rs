#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Failed to commit DB Update: `{0}`")]
    CommitError(#[from] sp_database::error::DatabaseError),
    #[error("Failed to decode DB Data: `{0}`")]
    ScaleCodecError(#[from] parity_scale_codec::Error),
    #[error("Failed to build Uuid: `{0}`")]
    Uuid(#[from] uuid::Error),
    #[error("A value was queryied that was not initialized at column: `{0}` key: `{1}`")]
    ValueNotInitialized(u32, String),
    #[error("The data stored at column `{0}` key: `{1}`, has been corrupted")]
    CorruptedValue(u32, String),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
