/// Define traits related to hash functions.
pub mod hash;

/// Define traits related to transaction.
pub mod transaction;

/// A trait for types that can be shared between threads + copied.
pub trait ThreadSafeCopy: Send + Sync + Copy + 'static {}
