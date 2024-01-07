#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Failed to commit DB Update: `{0}`")]
    CommitError(#[from] sp_database::error::DatabaseError),
    #[error("Failed to deserialize DB Data: `{0}`")]
    DeserializeError(#[from] parity_scale_codec::Error),
}
