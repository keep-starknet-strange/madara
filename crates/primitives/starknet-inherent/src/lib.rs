//! The address of the account receiving the network fee
use std::num::NonZeroU128;

use blockifier::blockifier::block::GasPrices;
use parity_scale_codec::{Decode, Encode};
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError};
use thiserror::Error;

/// The identifier for the `sequencer_address` inherent.
pub const STARKNET_INHERENT_IDENTIFIER: InherentIdentifier = *b"starknet";

/// Default value in case the sequencer address is not set.
pub const DEFAULT_SEQUENCER_ADDRESS: [u8; 32] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 222, 173];

/// The storage key for the sequencer address value.
pub const SEQ_ADDR_STORAGE_KEY: &[u8] = b"starknet::seq_addr";

#[derive(Clone, sp_core::RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct L1GasPrices {
    pub eth_l1_gas_price: NonZeroU128,       // In wei.
    pub strk_l1_gas_price: NonZeroU128,      // In fri.
    pub eth_l1_data_gas_price: NonZeroU128,  // In wei.
    pub strk_l1_data_gas_price: NonZeroU128, // In fri.
    pub last_update_timestamp: u128,
}

impl Default for L1GasPrices {
    fn default() -> Self {
        L1GasPrices {
            eth_l1_gas_price: NonZeroU128::new(1).unwrap(),
            strk_l1_gas_price: NonZeroU128::new(1).unwrap(),
            eth_l1_data_gas_price: NonZeroU128::new(1).unwrap(),
            strk_l1_data_gas_price: NonZeroU128::new(1).unwrap(),
            last_update_timestamp: Default::default(),
        }
    }
}

#[derive(Error, sp_core::RuntimeDebug)]
pub enum L1GasPriceError {
    #[error("Failed to convert {0} with value {1}  to NonZeroU128")]
    GasPriceCoversionError(String, u128),
}

impl From<L1GasPrices> for GasPrices {
    fn from(l1_gas_prices: L1GasPrices) -> Self {
        GasPrices {
            eth_l1_gas_price: l1_gas_prices.eth_l1_gas_price,
            strk_l1_gas_price: l1_gas_prices.strk_l1_gas_price,
            eth_l1_data_gas_price: l1_gas_prices.eth_l1_data_gas_price,
            strk_l1_data_gas_price: l1_gas_prices.strk_l1_data_gas_price,
        }
    }
}

/// Struct to hold all data for the Starknet inherent
#[derive(Clone, sp_core::RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct StarknetInherentData {
    /// The sequencer address field.
    pub sequencer_address: [u8; 32],
    /// The L1 gas price
    pub l1_gas_price: L1GasPrices,
}

/// The inherent type for the sequencer address.
pub type InherentType = StarknetInherentData;

#[derive(Error, sp_core::RuntimeDebug)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
/// Error types when working with the sequencer address.
pub enum InherentError {
    /// Submitted address must be `[u8; 32]`.
    #[error("Inherent decoding error")]
    WrongAddressFormat,
}

impl IsFatalError for InherentError {
    fn is_fatal_error(&self) -> bool {
        match self {
            InherentError::WrongAddressFormat => true,
        }
    }
}

#[cfg(feature = "client")]
mod reexport_for_client_only {
    use std::array::TryFromSliceError;
    use std::boxed::Box;

    use parity_scale_codec::{Decode, Encode};

    use super::*;

    impl InherentError {
        /// Try to create an instance ouf of the given identifier and data.
        // TODO: Bad name. This let think that it uses the trait TryFrom
        pub fn try_from(id: &InherentIdentifier, mut data: &[u8]) -> Option<Self> {
            if id == &STARKNET_INHERENT_IDENTIFIER { <InherentError as Decode>::decode(&mut data).ok() } else { None }
        }
    }

    #[derive(Clone, Decode, Encode, sp_core::RuntimeDebug)]
    /// The inherent data provider for sequencer address.
    pub struct InherentDataProvider {
        pub starknet_inherent_data: InherentType,
    }

    impl InherentDataProvider {
        /// Create `Self` using the given `addr`.
        pub fn new(data: InherentType) -> Self {
            Self { starknet_inherent_data: data }
        }

        /// Returns the sequencer address of this inherent data provider.
        pub fn starknet_inherent_data(&self) -> &InherentType {
            &self.starknet_inherent_data
        }
    }

    #[async_trait::async_trait]
    impl sp_inherents::InherentDataProvider for InherentDataProvider {
        async fn provide_inherent_data(&self, inherent_data: &mut InherentData) -> Result<(), sp_inherents::Error> {
            inherent_data.put_data(STARKNET_INHERENT_IDENTIFIER, &self.starknet_inherent_data)
        }

        async fn try_handle_error(
            &self,
            identifier: &InherentIdentifier,
            error: &[u8],
        ) -> Option<Result<(), sp_inherents::Error>> {
            Some(Err(sp_inherents::Error::Application(Box::from(InherentError::try_from(identifier, error)?))))
        }
    }
}

#[cfg(feature = "client")]
pub use reexport_for_client_only::*;
