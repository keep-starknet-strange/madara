#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Failed to commit DB Update: `{0}`")]
    CommitError(#[from] sp_database::error::DatabaseError),
    #[error("Failed to deserialize DB Data: `{0}`")]
    DeserializeError(#[from] parity_scale_codec::Error),
    #[error("Failed to build Uuid: `{0}`")]
    Uuid(#[from] uuid::Error),
    #[error("A value was queryied that was not initialized at column: `{0}` key: `{1}`")]
    ValueNotInitialized(u32, String),
}
