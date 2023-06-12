use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::mem;

use blockifier::execution::contract_class::ContractClass;
use cairo_vm::felt::Felt252;
use cairo_vm::serde::deserialize_program::{parse_program, parse_program_json, ProgramJson, ReferenceManager};
use cairo_vm::types::program::{Program, SharedProgramData};
use derive_more::Constructor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointType};
use starknet_api::stdlib::collections::HashMap;

use super::entrypoint_wrapper::{EntryPointTypeWrapper, EntryPointWrapper};
use crate::alloc::string::ToString;
use crate::scale_codec::{Decode, Encode, Error, Input, MaxEncodedLen, Output};
use crate::scale_info::build::Fields;
use crate::scale_info::{Path, Type, TypeInfo};
/// Max number of entrypoints types (EXTERNAL/L1_HANDLER/CONSTRUCTOR)
/// Converts the program type from SN API into a Cairo VM-compatible type.
pub fn deserialize_program_wrapper<'de, D: Deserializer<'de>>(deserializer: D) -> Result<ProgramWrapper, D::Error> {
    Ok(ProgramJson::deserialize(deserializer)?.into())
}
/// Helper function to serialize a [ProgramWrapper]. This function uses the [Serialize] function
/// from [ProgramJson]
fn serialize_program_wrapper<S: Serializer>(v: &ProgramWrapper, serializer: S) -> Result<S::Ok, S::Error> {
    let v: ProgramJson = v.clone().into();
    v.serialize(serializer)
}

/// Contract Class type wrapper.
#[derive(Clone, Debug, PartialEq, Eq, TypeInfo, Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ContractClassWrapper {
    /// Wrapper type for a [Program] object. (It's not really a wrapper it's a copy of the type but
    /// we implement the necessary traits.)
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "deserialize_program_wrapper", serialize_with = "serialize_program_wrapper")
    )]
    pub program: ProgramWrapper,
    /// Wrapper type for a [HashMap<String, EntryPoint>] object. (It's not really a wrapper it's a
    /// copy of the type but we implement the necessary traits.)
    pub entry_points_by_type: EntrypointMapWrapper,
}

impl From<ContractClassWrapper> for ContractClass {
    fn from(value: ContractClassWrapper) -> Self {
        Self {
            program: value.program.into(),
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
/// SCALE trait.
impl MaxEncodedLen for ContractClassWrapper {
    fn max_encoded_len() -> usize {
        20971520
    }
}

/// Wrapper type for a [HashMap<String, EntryPoint>] object. (It's not really a wrapper it's a
/// copy of the type but we implement the necessary traits.)
#[derive(Clone, Debug, PartialEq, Eq, Default, Constructor)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct EntrypointMapWrapper(pub HashMap<EntryPointTypeWrapper, Vec<EntryPointWrapper>>);
#[derive(Clone, Debug, PartialEq, Eq, Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
struct TupleTemp(EntryPointTypeWrapper, Vec<EntryPointWrapper>);

impl From<(EntryPointTypeWrapper, Vec<EntryPointWrapper>)> for TupleTemp {
    fn from(value: (EntryPointTypeWrapper, Vec<EntryPointWrapper>)) -> Self {
        Self(value.0, value.1)
    }
}
impl From<TupleTemp> for (EntryPointTypeWrapper, Vec<EntryPointWrapper>) {
    fn from(value: TupleTemp) -> Self {
        (value.0, value.1)
    }
}

/// SCALE trait.
impl Encode for EntrypointMapWrapper {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        let val: Vec<TupleTemp> = self.0.clone().into_iter().map(|val| val.into()).collect();
        dest.write(&Encode::encode(&val));
    }
}
/// SCALE trait.
impl Decode for EntrypointMapWrapper {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let val: Vec<(EntryPointTypeWrapper, Vec<EntryPointWrapper>)> =
            Decode::decode(input).map_err(|_| Error::from("Can't get EntrypointMap from input buffer."))?;
        Ok(EntrypointMapWrapper(HashMap::from_iter(val.into_iter())))
    }
}

/// SCALE trait.
impl TypeInfo for EntrypointMapWrapper {
    type Identity = Self;

    // The type info is saying that the field element must be seen as an
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
impl From<ProgramJson> for ProgramWrapper {
    fn from(value: ProgramJson) -> Self {
        parse_program_json(value, None).unwrap().into()
    }
}

/// SCALE trait.
impl Encode for ProgramWrapper {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        let program_bytes = &Into::<Program>::into(self.clone()).to_bytes();
        let program_len = program_bytes.len() as u128;
        assert_eq!(program_len.to_be_bytes().len(), 16);

        dest.write(&program_len.to_be_bytes());
        dest.write(program_bytes);
    }
}

/// SCALE trait.
impl Decode for ProgramWrapper {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let mut buf: [u8; 16] = [0; 16];
        input.read(&mut buf)?;
        let size = u128::from_be_bytes(buf);
        let mut program_buf = vec![0u8; size as usize];
        input.read(program_buf.as_mut_slice())?;
        let program = Program::from_bytes(&program_buf, None)
            .map_err(|e| Error::from("Can't get Program from input buffer.").chain(e.to_string()))?;
        Ok(program.into())
    }
}

/// SCALE trait.
impl TypeInfo for ProgramWrapper {
    type Identity = Self;

    // The type info is saying that the field element must be seen as an
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
