use scale_codec::{Decode, Encode};
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError};

/// The identifier for the `sequencer_address` inherent.
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"seqaddr0";

pub const SEQUENCER_ADDRESS: [u8; 32] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 222, 173];

pub const TEST_ADDRESS: [u8; 32] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 222, 178];

pub const OTHER_TEST_ADDRESS: [u8; 32] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 224, 179];

/// The inherent type for the sequencer address. 
pub type InherentType = [u8; 32];

#[derive(Decode, Encode, sp_runtime::RuntimeDebug)]
pub enum InherentError {
    /// Submitted address must be `[u8; 32]`.
    WrongAddressFormat,
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		match self {
			InherentError::WrongAddressFormat => true,
		}
	}
}

impl InherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, mut data: &[u8]) -> Option<Self> {
		if id == &INHERENT_IDENTIFIER {
			<InherentError as Decode>::decode(&mut data).ok()
		} else {
			None
		}
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
pub struct InherentDataProvider {
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
    pub fn from_const() -> Self {
        InherentDataProvider {sequencer_address: TEST_ADDRESS}
    }

    /// Default address if sequencer address is not specified. 
    pub fn from_other_const() -> Self {
        InherentDataProvider {sequencer_address: OTHER_TEST_ADDRESS}
    }

    pub fn from_vec(storage_val: Vec<u8>) -> Self {
        let addr: [u8; 32] = slice_to_arr(&storage_val);
        InherentDataProvider {sequencer_address: addr}
    }
}
 /// Helper function to convert storage value. 
pub fn slice_to_arr(s: &[u8]) -> [u8; 32] {
    if s.len() == 0 { 
        SEQUENCER_ADDRESS 
    } else { OTHER_TEST_ADDRESS }
    //s.try_into().unwrap_or(SEQUENCER_ADDRESS)
}

#[cfg(feature = "std")]
#[async_trait::async_trait]
impl sp_inherents::InherentDataProvider for InherentDataProvider {
    async fn provide_inherent_data(
        &self, 
        inherent_data: &mut InherentData
    ) -> Result<(), sp_inherents::Error> {
        inherent_data.put_data(INHERENT_IDENTIFIER, &self.sequencer_address)
    }

    async fn try_handle_error(
		&self,
		_identifier: &InherentIdentifier,
		_error: &[u8],
	) -> Option<Result<(), sp_inherents::Error>> {
		None
	}
}