/// Define traits related to hash functions.
pub mod hash;

/// A trait for types that can be shared between threads + copied.
pub trait SendSyncStatic: Send + Sync + 'static {}
