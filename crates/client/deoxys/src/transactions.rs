use std::{default, f32::consts::E};

use mp_starknet::{transaction::types::{Transaction, MaxArraySize, TxType}, execution::{types::{Felt252Wrapper, EntryPointTypeWrapper, ContractAddressWrapper, CallEntryPointWrapper, MaxCalldataSize, ClassHashWrapper}, felt252_wrapper}};
use sp_core::{bounded_vec::BoundedVec, U256};
use blockifier::execution::contract_class::ContractClass;
use starknet_client::reader::objects::transaction::{IntermediateInvokeTransaction, IntermediateDeclareTransaction, DeployAccountTransaction, L1HandlerTransaction};
use starknet_api::hash::StarkFelt;

pub fn declare_tx_to_starknet_tx(declare_transaction: IntermediateDeclareTransaction) -> Transaction {

    let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
    for item in &declare_transaction.signature.0 {
        match signature_vec.try_push(Felt252Wrapper::try_from(item.bytes()).unwrap()) {
            Ok(_) => {},
            Err(_) => {
                panic!("Signature too long");
            }
        }
        signature_vec.try_push(Felt252Wrapper::try_from(item.bytes()).unwrap());
    }
    let calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();

    let call_entry_point = CallEntryPointWrapper::new(
        Some(Felt252Wrapper::try_from(declare_transaction.class_hash.0.bytes()).unwrap()),   //class_hash: Option<ClassHashWrapper>,
        EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
        Some(Felt252Wrapper::default()),
        calldata_vec,
        ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
        ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
        Felt252Wrapper::ZERO,
        Some(ClassHashWrapper::ZERO)
    );
    
    Transaction {
        tx_type: TxType::Declare,
        version: b'1',
        hash: Felt252Wrapper(declare_transaction.transaction_hash.0.into()),
        signature: signature_vec,
        sender_address: Felt252Wrapper(declare_transaction.sender_address.0.try_into().unwrap().bytes()),
        nonce: Felt252Wrapper::try_from(declare_transaction.nonce.0.bytes()).unwrap(),
        call_entrypoint: call_entry_point,
        contract_class: Option::<ContractClass>::default(),
        contract_address_salt: Option::<U256>::default(),
        max_fee: Felt252Wrapper::from(declare_transaction.max_fee.0),
        is_query: false, // Assuming default value
    }
}


pub fn invoke_tx_to_starknet_tx(invoke_transaction : IntermediateInvokeTransaction) -> Transaction {
    /*let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
            for item in invoke_transaction.signature {
                match signature_vec.try_push(Felt252Wrapper::try_from(item).unwrap()) {
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Signature too long");
                    }
                }
                //signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            }*/
    let version_byte: [u8; 32] = match Felt252Wrapper::try_from(invoke_transaction.version.0.into()) {
        Ok(valeur) => {
            Felt252Wrapper::from(valeur).into()
        },
        Err(_) => {panic!("Version too long")}
    };
    let version_u8: u8 = match version_byte[0] {
        0 => 0,
        1 => 1,
        01 => 2,
        _ => panic!("Version not supported")
    };
    let tx = Transaction {
        tx_type: TxType::Invoke,
        version: version_u8,
        hash: Felt252Wrapper::try_from(invoke_transaction.transaction_hash.0.as_be_bytes()).unwrap(),
        signature: default,
        sender_address: Felt252Wrapper::try_from(invoke_transaction.contract_address.get().as_be_bytes()).unwrap(),
        nonce: ContractAddressWrapper::try_from(invoke_transaction.nonce.0.as_be_bytes()).unwrap(),
        call_entrypoint: CallEntryPointWrapper::default(),
        contract_class: Option::<ContractClass>::default(),
        contract_address_salt: Option::<U256>::default(),
        max_fee: Felt252Wrapper::ONE,
        is_query: todo!(),
    };
}

pub fn deploy_account_tx_to_starknet_tx(mut deploy_account_transaction : DeployAccountTransaction) -> Transaction {
    let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
    for item in &deploy_account_transaction.signature.0 {
        match signature_vec.try_push(Felt252Wrapper::try_from(item.bytes()).unwrap()) {
            Ok(_) => {},
            Err(_) => {
                panic!("Signature too long");
            }
        }
        signature_vec.try_push(Felt252Wrapper::try_from(item.bytes()).unwrap());
    }
    let calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();

    let call_entry_point = CallEntryPointWrapper::new(
        Some(Felt252Wrapper::try_from(deploy_account_transaction.class_hash.0.bytes()).unwrap()),   //class_hash: Option<ClassHashWrapper>,
        EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
        Some(Felt252Wrapper::default()),
        calldata_vec,
        ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
        ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
        Felt252Wrapper::ZERO,
        Some(ClassHashWrapper::ZERO)
    );
    
    Transaction {
        tx_type: TxType::Declare,
        version: b'1',
        hash: Felt252Wrapper(deploy_account_transaction.transaction_hash.0.into()),
        signature: signature_vec,
        sender_address: Felt252Wrapper(deploy_account_transaction.sender_address.0.try_into().unwrap().bytes()),
        nonce: Felt252Wrapper::try_from(deploy_account_transaction.nonce.0.bytes()).unwrap(),
        call_entrypoint: call_entry_point,
        contract_class: Option::<ContractClass>::default(),
        contract_address_salt: Option::<U256>::default(),
        max_fee: Felt252Wrapper::from(deploy_account_transaction.max_fee.0),
        is_query: false, // Assuming default value
    }
}

pub fn l1handler_tx_to_starknet_tx(mut l1handler_transaction : L1HandlerTransaction) -> Transaction {
    let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
    for item in &l1handler_transaction.signature.0 {
        match signature_vec.try_push(Felt252Wrapper::try_from(item.bytes()).unwrap()) {
            Ok(_) => {},
            Err(_) => {
                panic!("Signature too long");
            }
        }
        signature_vec.try_push(Felt252Wrapper::try_from(item.bytes()).unwrap());
    }
    let calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();

    let call_entry_point = CallEntryPointWrapper::new(
        Some(Felt252Wrapper::try_from(l1handler_transaction.class_hash.0.bytes()).unwrap()),   //class_hash: Option<ClassHashWrapper>,
        EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
        Some(Felt252Wrapper::default()),
        calldata_vec,
        ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
        ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
        Felt252Wrapper::ZERO,
        Some(ClassHashWrapper::ZERO)
    );
    
    Transaction {
        tx_type: TxType::Declare,
        version: b'1',
        hash: Felt252Wrapper(l1handler_transaction.transaction_hash.0.into()),
        signature: signature_vec,
        sender_address: Felt252Wrapper(l1handler_transaction.sender_address.0.try_into().unwrap().bytes()),
        nonce: Felt252Wrapper::try_from(l1handler_transaction.nonce.0.bytes()).unwrap(),
        call_entrypoint: call_entry_point,
        contract_class: Option::<ContractClass>::default(),
        contract_address_salt: Option::<U256>::default(),
        max_fee: Felt252Wrapper::from(l1handler_transaction.max_fee.0),
        is_query: false, // Assuming default value
    }
}
