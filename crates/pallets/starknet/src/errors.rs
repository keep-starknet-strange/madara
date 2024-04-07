use blockifier::execution::errors::{EntryPointExecutionError, PreExecutionError};
use blockifier::transaction::errors::TransactionExecutionError;
use frame_support::traits::PalletError;
use parity_scale_codec::{Decode, Encode, Output};
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
        // read the first byte to decode the index,
        // the index will decide the type of error
        let index = input.read_byte()?;

        // convert bytes into string and then parse the string back into the error
        // create a new error depending on the string received
        let mut buffer: Vec<u8> = Vec::new();

        //skip the first byte
        input.read(&mut buffer)?;

        let message = String::from_utf8(buffer).unwrap();

        //convert the vector bytes back into blockifier error
        // create a mini parser that decodes

        Ok(match index {
            0 => BlockifierErrors::EntryPointExecutionError(message),
            1 => BlockifierErrors::PreExecutionError(message),
            2 => BlockifierErrors::TransactionExecutionError(message),
            _ => return Err("Invalid variant index".into()),
        })
    }
}

// encode errors into a vector of bytes
impl Encode for BlockifierErrors {
    fn size_hint(&self) -> usize {
        // an extra byte for padding
        self.encode().len() + 1
    }

    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        let index: u8 = match self {
            BlockifierErrors::EntryPointExecutionError(_) => 0,
            BlockifierErrors::PreExecutionError(_) => 1,
            BlockifierErrors::TransactionExecutionError(_) => 2,
        };

        dest.push_byte(index);

        let encoded_message = match self {
            BlockifierErrors::EntryPointExecutionError(err) => err.to_string().as_bytes(),
            BlockifierErrors::PreExecutionError(err) => err.to_string().as_bytes(),
            BlockifierErrors::TransactionExecutionError(err) => err.to_string().as_bytes(),
        };

        dest.write(encoded_message)
    }

    fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(self.size_hint());
        self.encode_to(&mut encoded);
        encoded
    }

    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        let mut encoded = Vec::with_capacity(self.size_hint());
        self.encode_to(&mut encoded);
        f(&encoded)
    }
    fn encoded_size(&self) -> usize {
        self.size_hint()
    }
}

impl PalletError for BlockifierErrors {
    // the largest size available
    const MAX_ENCODED_SIZE: usize = 17;
}
