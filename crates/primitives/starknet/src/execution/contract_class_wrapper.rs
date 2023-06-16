use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::mem;

use blockifier::execution::contract_class::ContractClass;
use cairo_vm::felt::Felt252;
use cairo_vm::serde::deserialize_program::{parse_program, parse_program_json, ProgramJson, ReferenceManager};
use cairo_vm::types::errors::program_errors::ProgramError;
use cairo_vm::types::program::{Program, SharedProgramData};
use derive_more::Constructor;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointType};
use starknet_api::stdlib::collections::HashMap;

use super::entrypoint_wrapper::{EntryPointTypeWrapper, EntryPointWrapper};
use crate::alloc::string::ToString;
use crate::scale_codec::{Decode, Encode, Error, Input, MaxEncodedLen, Output};
use crate::scale_info::build::Fields;
use crate::scale_info::{Path, Type, TypeInfo};

impl Serialize for ProgramWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let program_json: ProgramJson = self.clone().into();
        program_json.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for ProgramWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        ProgramJson::deserialize(deserializer)?
            .try_into()
            .map_err(|e| de::Error::custom(format!("couldn't convert programjson to program wrapper {e:}")))
    }
}

/// [ContractClass] type equivalent. This is not really a wrapper it's more of a copy where we
/// implement the substrate necessary traits.
#[derive(Clone, Debug, PartialEq, Eq, TypeInfo, Default, Encode, Decode, Serialize, Deserialize)]
pub struct ContractClassWrapper {
    /// Wrapper type for a [Program] object. (It's not really a wrapper it's a copy of the type but
    /// we implement the necessary traits.)
    pub program: ProgramWrapper,
    /// Wrapper type for a [HashMap<String, EntryPoint>] object. (It's not really a wrapper it's a
    /// copy of the type but we implement the necessary traits.)
    pub entry_points_by_type: EntrypointMapWrapper,
}

impl ContractClassWrapper {
    // This is the maximum size of a contract in starknet. https://docs.starknet.io/documentation/starknet_versions/limits_and_triggers/
    const MAX_CONTRACT_BYTE_SIZE: usize = 20971520;
}

impl From<ContractClassWrapper> for ContractClass {
    fn from(value: ContractClassWrapper) -> Self {
        Self {
            program: value.program.into(),
            // Convert EntrypointMapWrapper to HashMap<EntryPointType, Vec<EntryPoint>>
            entry_points_by_type: HashMap::from_iter(value.entry_points_by_type.0.iter().clone().map(
                |(entrypoint_type, entrypoints)| {
                    (entrypoint_type.clone().into(), entrypoints.clone().into_iter().map(|val| val.into()).collect())
                },
            )),
        }
    }
}

impl From<ContractClass> for ContractClassWrapper {
    fn from(value: ContractClass) -> Self {
        Self {
            program: value.program.into(),
            entry_points_by_type: EntrypointMapWrapper(unsafe {
                mem::transmute::<
                    HashMap<EntryPointType, Vec<EntryPoint>>,
                    HashMap<EntryPointTypeWrapper, Vec<EntryPointWrapper>>,
                >(value.entry_points_by_type)
            }),
        }
    }
}

impl MaxEncodedLen for ContractClassWrapper {
    fn max_encoded_len() -> usize {
        // This is the maximum size of a contract in starknet. https://docs.starknet.io/documentation/starknet_versions/limits_and_triggers/
        Self::MAX_CONTRACT_BYTE_SIZE
    }
}

/// Wrapper type for a [HashMap<String, EntryPoint>] object. (It's not really a wrapper it's a
/// copy of the type but we implement the necessary traits.)
#[derive(Clone, Debug, PartialEq, Eq, Default, Constructor, Serialize, Deserialize)]
pub struct EntrypointMapWrapper(pub HashMap<EntryPointTypeWrapper, Vec<EntryPointWrapper>>);

/// SCALE trait.
impl Encode for EntrypointMapWrapper {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        // Convert the EntrypointMapWrapper to Vec<(EntryPointTypeWrapper, Vec<EntryPointWrapper>)> to be
        // able to use the Encode trait from this type. We implemented it for EntryPointWrapper, derived it
        // for EntryPointTypeWrapper so we can use it for Vec<(EntryPointTypeWrapper,
        // Vec<EntryPointWrapper>)>.
        let val: Vec<(EntryPointTypeWrapper, Vec<EntryPointWrapper>)> = self.0.clone().into_iter().collect();
        dest.write(&Encode::encode(&val));
    }
}
/// SCALE trait.
impl Decode for EntrypointMapWrapper {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        // Convert the EntrypointMapWrapper to Vec<(EntryPointTypeWrapper, Vec<EntryPointWrapper>)> to be
        // able to use the Decode trait from this type. We implemented it for EntryPointWrapper, derived it
        // for EntryPointTypeWrapper so we can use it for Vec<(EntryPointTypeWrapper,
        // Vec<EntryPointWrapper>)>.
        let val: Vec<(EntryPointTypeWrapper, Vec<EntryPointWrapper>)> =
            Decode::decode(input).map_err(|_| Error::from("Can't get EntrypointMap from input buffer."))?;
        Ok(EntrypointMapWrapper(HashMap::from_iter(val.into_iter())))
    }
}

/// SCALE trait.
impl TypeInfo for EntrypointMapWrapper {
    type Identity = Self;

    // The type info is saying that the EntryPointByType must be seen as an
    // array of bytes.
    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("EntrypointMapWrapper", module_path!()))
            .composite(Fields::unnamed().field(|f| f.ty::<[u8]>().type_name("EntrypointMap")))
    }
}

/// Wrapper type for a [Program] object. (It's not really a wrapper it's a copy of the type but
/// we implement the necessary traits.)
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ProgramWrapper {
    /// Fields contained in the program object.
    pub shared_program_data: Arc<SharedProgramData>,
    /// Constants of the program.
    pub constants: HashMap<String, Felt252>,
    /// All the references of the program.
    pub reference_manager: ReferenceManager,
}

impl From<Program> for ProgramWrapper {
    fn from(value: Program) -> Self {
        Self {
            shared_program_data: value.shared_program_data,
            constants: value.constants,
            reference_manager: value.reference_manager,
        }
    }
}

impl From<ProgramWrapper> for Program {
    fn from(value: ProgramWrapper) -> Self {
        Self {
            shared_program_data: value.shared_program_data,
            constants: value.constants,
            reference_manager: value.reference_manager,
        }
    }
}

impl From<ProgramWrapper> for ProgramJson {
    fn from(value: ProgramWrapper) -> Self {
        parse_program(value.into())
    }
}
impl TryFrom<ProgramJson> for ProgramWrapper {
    fn try_from(value: ProgramJson) -> Result<ProgramWrapper, ProgramError> {
        Ok(parse_program_json(value, None)?.into())
    }

    type Error = ProgramError;
}

/// SCALE trait.
impl Encode for ProgramWrapper {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        // Get the program to bytes.
        let program_bytes = &Into::<Program>::into(self.clone()).to_bytes();
        // Get the program bytes length to be able to decode it. We convert it to u128 to have a fix bytes
        // size so when we decode it we know that the first 16 bytes correspond to the program encoded size.
        let program_len = program_bytes.len() as u128;
        assert_eq!(program_len.to_be_bytes().len(), 16);

        dest.write(&program_len.to_be_bytes());
        dest.write(program_bytes);
    }
}

/// SCALE trait.
impl Decode for ProgramWrapper {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        // Get the program encoded length. We encoded the bytes length as u128 to be sure that the 16 first
        // bytes would be its length.
        let mut buf: [u8; 16] = [0; 16];
        input.read(&mut buf)?;
        let size = u128::from_be_bytes(buf);
        // Create a buffer of the size of the program.
        let mut program_buf = vec![0u8; size as usize];
        // Fill it with the program.
        input.read(program_buf.as_mut_slice())?;
        // Convert the program to bytes.
        let program = Program::from_bytes(&program_buf, None)
            .map_err(|e| Error::from("Can't get Program from input buffer.").chain(e.to_string()))?;
        Ok(program.into())
    }
}

/// SCALE trait.
impl TypeInfo for ProgramWrapper {
    type Identity = Self;

    // The type info is saying that the `ProgramWrapper` must be seen as an
    // array of bytes.
    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("ProgramWrapper", module_path!()))
            .composite(Fields::unnamed().field(|f| f.ty::<[u8]>().type_name("Program")))
    }
}

#[cfg(test)]
mod tests {
    use blockifier::execution::contract_class::ContractClass;

    use super::*;

    pub fn get_contract_class(contract_content: &'static [u8]) -> ContractClass {
        serde_json::from_slice(contract_content).unwrap()
    }

    #[test]
    fn test_serialize_deserialize_contract_class() {
        let contract_class: ContractClassWrapper =
            get_contract_class(include_bytes!("../../../../../cairo-contracts/build/NoValidateAccount.json")).into();
        let contract_class_serialized = serde_json::to_vec(&contract_class).unwrap();
        let contract_class_deserialized: ContractClassWrapper =
            serde_json::from_slice(&contract_class_serialized).unwrap();

        assert_eq!(contract_class, contract_class_deserialized);
    }

    #[test]
    fn test_encode_decode_contract_class() {
        let contract_class: ContractClassWrapper =
            get_contract_class(include_bytes!("../../../../../cairo-contracts/build/NoValidateAccount.json")).into();
        let encoded = contract_class.encode();
        assert_eq!(contract_class, ContractClassWrapper::decode(&mut &encoded[..]).unwrap())
    }
}
