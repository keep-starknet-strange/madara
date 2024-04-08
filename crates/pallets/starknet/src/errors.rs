use frame_support::traits::PalletError;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

// Wrapper Type For Blockifier Errors
#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct BlockifierErrors(pub String);

impl PalletError for BlockifierErrors {
    // max value allowed by MAX_MODULE_ERROR_ENCODED_SIZE
    const MAX_ENCODED_SIZE: usize = 3;
}
