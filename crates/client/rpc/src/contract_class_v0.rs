use anyhow::{anyhow, Result};
use blockifier::execution::contract_class::ContractClassV0;
use cairo_vm::serde::deserialize_program::ProgramJson;
use indexmap::IndexMap;
use mp_felt::Felt252Wrapper;
use mp_transactions::V0ContractClassData;
use serde_json::value::RawValue;
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointType};
use starknet_core::types::contract::legacy::{
    LegacyApTrackingData, LegacyAttribute, LegacyFlowTrackingData, LegacyHint, LegacyIdentifier,
    LegacyIdentifierMember, LegacyProgram, LegacyReference, LegacyReferenceManager,
};
use starknet_core::types::{
    CompressedLegacyContractClass, ContractClass, LegacyContractEntryPoint, LegacyEntryPointsByType,
};
use starknet_ff::FromByteArrayError;

pub fn casm_contract_class_to_compressed_legacy_contract_class(
    contract_class: ContractClassV0,
    mut v0_contract_class_data: V0ContractClassData,
) -> Result<ContractClass> {
    let program_json = ProgramJson::from(contract_class.program.clone());
    let legacy_program = LegacyProgram {
        // If we stored some accessible_scopes, it means attributes was `Some`
        attributes: v0_contract_class_data.accessible_scopes.map(|accessible_scopes| {
            program_json
                .attributes
                .into_iter()
                .zip(accessible_scopes)
                .map(|(attribute, accessible_scopes)| LegacyAttribute {
                    accessible_scopes,
                    end_pc: attribute.end_pc.try_into().expect("the value shoud have fit"),
                    flow_tracking_data: attribute.flow_tracking_data.map(|ftd| LegacyFlowTrackingData {
                        ap_tracking: LegacyApTrackingData {
                            group: ftd.ap_tracking.group.try_into().expect("the value shoud have fit"),
                            offset: ftd.ap_tracking.offset.try_into().expect("the value shoud have fit"),
                        },
                        reference_ids: ftd
                            .reference_ids
                            .into_iter()
                            .map(|(k, v)| (k, v.try_into().expect("the value should have fit")))
                            .collect(),
                    }),
                    name: attribute.name,
                    start_pc: attribute.start_pc.try_into().expect("value should have fit"),
                    value: attribute.value,
                })
                .collect()
        }),
        builtins: program_json
            .builtins
            .into_iter()
            .map(|bn| {
                bn.name().strip_suffix("_builtin").expect("all builtins names end with this suffix atm").to_string()
            })
            .collect(),
        compiler_version: v0_contract_class_data.compiler_version,
        data: program_json
            .data
            .into_iter()
            .map(|mr| match mr {
                cairo_vm::types::relocatable::MaybeRelocatable::RelocatableValue(_) => {
                    panic!("All values should be relocatable at this point")
                }
                cairo_vm::types::relocatable::MaybeRelocatable::Int(f) => Felt252Wrapper::from(f).into(),
            })
            .collect(),
        // I'm pretty sure DeclareTransaction are guaranteed to not contain any program debug info
        debug_info: None,
        hints: program_json
            .hints
            .into_iter()
            .map(|(k, v)| {
                (
                    k.try_into().expect("the value should have fit"),
                    v.into_iter()
                        .map(|hp| LegacyHint {
                            accessible_scopes: hp.accessible_scopes,
                            code: hp.code,
                            flow_tracking_data: LegacyFlowTrackingData {
                                ap_tracking: LegacyApTrackingData {
                                    group: hp
                                        .flow_tracking_data
                                        .ap_tracking
                                        .group
                                        .try_into()
                                        .expect("value should have fit"),
                                    offset: hp
                                        .flow_tracking_data
                                        .ap_tracking
                                        .offset
                                        .try_into()
                                        .expect("value should have fit"),
                                },
                                reference_ids: hp
                                    .flow_tracking_data
                                    .reference_ids
                                    .into_iter()
                                    .map(|(k, v)| (k, v.try_into().expect("value should have fit")))
                                    .collect(),
                            },
                        })
                        .collect(),
                )
            })
            .collect(),
        identifiers: program_json
            .identifiers
            .into_iter()
            .map(|(k, identifier)| {
                let identifiers_data =
                    v0_contract_class_data.identifiers_data.remove(&k).expect("there should have been a value there");
                (
                    k,
                    LegacyIdentifier {
                        cairo_type: identifier.cairo_type,
                        full_name: identifier.full_name,
                        members: identifier.members.map(|members| {
                            members
                                .into_iter()
                                .map(|(k, member)| {
                                    (
                                        k,
                                        LegacyIdentifierMember {
                                            cairo_type: member.cairo_type,
                                            offset: member.offset.try_into().expect("value should have fit"),
                                        },
                                    )
                                })
                                .collect()
                        }),
                        pc: identifier.pc.map(|pc| pc.try_into().expect("value should have fit")),
                        r#type: identifier.type_.expect("there should have been a value there"),
                        value: identifier.value.map(|v| {
                            RawValue::from_string(v.to_string()).expect("the value should have been convertible")
                        }),
                        decorators: identifiers_data.decorators,
                        destination: identifiers_data.destination,
                        references: identifiers_data.references,
                        size: identifiers_data.size,
                    },
                )
            })
            .collect(),
        main_scope: v0_contract_class_data.main_scope,
        prime: program_json.prime,
        reference_manager: LegacyReferenceManager {
            references: program_json
                .reference_manager
                .references
                .into_iter()
                .zip(v0_contract_class_data.references_data)
                .map(|(reference, reference_data)| {
                    LegacyReference {
                        ap_tracking_data: LegacyApTrackingData {
                            group: reference.ap_tracking_data.group.try_into().expect("value should have fit"),
                            offset: reference.ap_tracking_data.offset.try_into().expect("value should have fit"),
                        },
                        pc: reference_data.pc,
                        // We have the value but not the method to recreate the string from it.
                        // So for now we store it as it is in the rpc, and get it back here.
                        value: reference_data.value,
                    }
                })
                .collect(),
        },
    };

    let entry_points_by_type = to_legacy_entry_points_by_type(&contract_class.entry_points_by_type)?;

    Ok(ContractClass::Legacy(CompressedLegacyContractClass {
        program: legacy_program.compress().unwrap(),
        entry_points_by_type,
        abi: v0_contract_class_data.abi,
    }))
}

/// Returns a [Result<LegacyEntryPointsByType>] (starknet-rs type) from a [HashMap<EntryPointType,
/// Vec<EntryPoint>>]
fn to_legacy_entry_points_by_type(
    entries: &IndexMap<EntryPointType, Vec<EntryPoint>>,
) -> Result<LegacyEntryPointsByType> {
    fn collect_entry_points(
        entries: &IndexMap<EntryPointType, Vec<EntryPoint>>,
        entry_point_type: EntryPointType,
    ) -> Result<Vec<LegacyContractEntryPoint>> {
        Ok(entries
            .get(&entry_point_type)
            .ok_or(anyhow!("Missing {:?} entry point", entry_point_type))?
            .iter()
            .map(|e| to_legacy_entry_point(e.clone()))
            .collect::<Result<Vec<LegacyContractEntryPoint>, FromByteArrayError>>()?)
    }

    let constructor = collect_entry_points(entries, EntryPointType::Constructor)?;
    let external = collect_entry_points(entries, EntryPointType::External)?;
    let l1_handler = collect_entry_points(entries, EntryPointType::L1Handler)?;

    Ok(LegacyEntryPointsByType { constructor, external, l1_handler })
}

/// Returns a [LegacyContractEntryPoint] (starknet-rs) from a [EntryPoint] (starknet-api)
fn to_legacy_entry_point(entry_point: EntryPoint) -> Result<LegacyContractEntryPoint, FromByteArrayError> {
    // let selector = FieldElement::from_bytes_be(&entry_point.selector.0.0)?;
    let selector = Felt252Wrapper::from(entry_point.selector).into();
    let offset = entry_point.offset.0;
    Ok(LegacyContractEntryPoint { selector, offset })
}
