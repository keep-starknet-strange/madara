use blockifier::execution::errors::{EntryPointExecutionError, PreExecutionError};
use blockifier::transaction::errors::TransactionExecutionError;
use frame_support::traits::PalletError;
use parity_scale_codec::{Decode, Encode};
use scale_info::build::{Fields, Variants};
use scale_info::{Path, Type, TypeInfo};

// Wrapper Type For Blockifier Errors
#[derive(Debug)]
pub enum BlockifierErrors {
    EntryPointExecutionError(EntryPointExecutionError),
    PreExecutionError(PreExecutionError),
    TransactionExecutionError(TransactionExecutionError),
}

impl TypeInfo for BlockifierErrors {
    // Since encoding and decoding as an array of bytes,
    // typeinfo will represent this information as an array of bytes
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder().path(Path::new("BlockifierErrors", module_path!())).variant(
            Variants::new()
                .variant("EntryPointExecutionError", |v| v.index(0).fields(Fields::unnamed().field(|f| f.ty::<[u8]>())))
                .variant("PreExecutionError", |v| v.index(1).fields(Fields::unnamed().field(|f| f.ty::<[u8]>())))
                .variant("TransactionExecutionError", |v| {
                    v.index(2).fields(Fields::unnamed().field(|f| f.ty::<[u8]>()))
                }),
        )
    }
}

// encode errors from vector of bytes back into enum
// https://github.com/paritytech/parity-scale-codec
impl Decode for BlockifierErrors {
    fn decode<I: parity_scale_codec::Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        todo!()
    }
}

// encode errors into a vector of bytes
impl Encode for BlockifierErrors {
    fn size_hint(&self) -> usize {
        match self {
            BlockifierErrors::EntryPointExecutionError(_) => 0,
            BlockifierErrors::PreExecutionError(_) => 0,
            BlockifierErrors::TransactionExecutionError(_) => 0,
        }
    }

    fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, dest: &mut T) {
        match self {
            BlockifierErrors::EntryPointExecutionError(_) => todo!(),
            BlockifierErrors::PreExecutionError(_) => todo!(),
            BlockifierErrors::TransactionExecutionError(_) => todo!(),
        }
    }

    fn encode(&self) -> Vec<u8> {
        match self {
            BlockifierErrors::EntryPointExecutionError(_) => todo!(),
            BlockifierErrors::PreExecutionError(_) => todo!(),
            BlockifierErrors::TransactionExecutionError(_) => todo!(),
        }
    }

    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        match self {
            BlockifierErrors::EntryPointExecutionError(_) => todo!(),
            BlockifierErrors::PreExecutionError(_) => todo!(),
            BlockifierErrors::TransactionExecutionError(_) => todo!(),
        }
    }

    fn encoded_size(&self) -> usize {
        match self {
            BlockifierErrors::EntryPointExecutionError(_) => todo!(),
            BlockifierErrors::PreExecutionError(_) => todo!(),
            BlockifierErrors::TransactionExecutionError(_) => todo!(),
        }
    }
}

impl PalletError for BlockifierErrors {
    // the largest size available
    const MAX_ENCODED_SIZE: usize = 17;
}
