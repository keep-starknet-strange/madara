/// General helper functions related to mocking
mod helpers;
pub use helpers::*;

/// Mock Runtime with default config
/// Closest to Public Starknet
pub mod default_mock;
pub use default_mock::*;

/// Mock Runtime with global state root enabled
pub mod state_root_mock;
