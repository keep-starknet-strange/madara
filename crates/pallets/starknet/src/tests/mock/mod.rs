/// General helper functions related to mocking
mod helpers;
pub use helpers::*;

/// Mock Runtime with default config
/// Closest to Public Starknet
pub mod setup_mock;
pub use setup_mock::*;
