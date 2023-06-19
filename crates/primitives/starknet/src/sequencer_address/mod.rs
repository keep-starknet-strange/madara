#![cfg_attr(not(feature = "std"), no_std)]

use scale_codec::{Decode, Encode};
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError};
use thiserror_no_std::Error;

/// The identifier for the `sequencer_address` inherent.
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"seqaddr0";

/// Default value in case the sequencer address is not set.
pub const DEFAULT_SEQUENCER_ADDRESS: [u8; 32] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 222, 173];

/// The storage key for the sequencer address value.
pub const SEQ_ADDR_STORAGE_KEY: &[u8] = b"starknet::seq_addr";

/// The inherent type for the sequencer address.
pub type InherentType = [u8; 32];

#[derive(Encode, sp_runtime::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Decode, Error))]
/// Error types when working with the sequencer address.
pub enum InherentError {
    /// Submitted address must be `[u8; 32]`.
    WrongAddressFormat,
}

#[cfg(feature = "std")]
impl std::fmt::Display for InherentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Inherent decoding error")
    }
}

impl IsFatalError for InherentError {
    fn is_fatal_error(&self) -> bool {
        match self {
            InherentError::WrongAddressFormat => true,
        }
    }
}

#[cfg(feature = "std")]
impl InherentError {
    /// Try to create an instance ouf of the given identifier and data.
    pub fn try_from(id: &InherentIdentifier, mut data: &[u8]) -> Option<Self> {
        if id == &INHERENT_IDENTIFIER { <InherentError as Decode>::decode(&mut data).ok() } else { None }
    }
}

/// Auxiliary trait to extract sequencer address inherent data.
pub trait SequencerAddressInherentData {
    /// Get sequencer address inherent data.
    fn sequencer_address_inherent_data(&self) -> Result<Option<InherentType>, sp_inherents::Error>;
}

impl SequencerAddressInherentData for InherentData {
    fn sequencer_address_inherent_data(&self) -> Result<Option<InherentType>, sp_inherents::Error> {
        self.get_data(&INHERENT_IDENTIFIER)
    }
}

#[derive(Copy, Clone, Decode, Encode, sp_runtime::RuntimeDebug)]
#[cfg(feature = "std")]
/// The inherent data provider for sequencer address.
pub struct InherentDataProvider {
    /// The sequencer address field.
    pub sequencer_address: InherentType,
}

#[cfg(feature = "std")]
impl InherentDataProvider {
    /// Create `Self` using the given `addr`.
    pub fn new(addr: InherentType) -> Self {
        Self { sequencer_address: addr }
    }

    /// Returns the sequencer address of this inherent data provider.
    pub fn sequencer_address(&self) -> InherentType {
        self.sequencer_address
    }

    /// Default address if sequencer address is not specified.
    pub fn from_default() -> Self {
        InherentDataProvider { sequencer_address: DEFAULT_SEQUENCER_ADDRESS }
    }

    /// Convert storage vector into `[u8; 32]`.
    pub fn from_vec(storage_val: Vec<u8>) -> Self {
        let addr: [u8; 32] = slice_to_arr(&storage_val);
        InherentDataProvider { sequencer_address: addr }
    }
}

/// Helper function to convert storage value.
pub fn slice_to_arr(slice: &[u8]) -> [u8; 32] {
    slice.try_into().unwrap_or(DEFAULT_SEQUENCER_ADDRESS)
}

#[cfg(feature = "std")]
#[async_trait::async_trait]
impl sp_inherents::InherentDataProvider for InherentDataProvider {
    async fn provide_inherent_data(&self, inherent_data: &mut InherentData) -> Result<(), sp_inherents::Error> {
        inherent_data.put_data(INHERENT_IDENTIFIER, &self.sequencer_address)
    }

    async fn try_handle_error(
        &self,
        identifier: &InherentIdentifier,
        error: &[u8],
    ) -> Option<Result<(), sp_inherents::Error>> {
        Some(Err(sp_inherents::Error::Application(Box::from(InherentError::try_from(identifier, error)?))))
    }
}
