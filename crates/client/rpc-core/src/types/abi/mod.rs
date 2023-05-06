mod contract_abi;
mod events_abi;
mod function_abi;

mod struct_abi;

pub use contract_abi::*;
pub use events_abi::*;
pub use function_abi::*;
pub use struct_abi::*;

use super::TypedParameter;

pub type ABI = String;
