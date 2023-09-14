use mp_starknet::{transaction::types::{Transaction, MaxArraySize, TxType}, execution::{types::{Felt252Wrapper, EntryPointTypeWrapper, ContractAddressWrapper, CallEntryPointWrapper, MaxCalldataSize, ClassHashWrapper}, felt252_wrapper}};
use sp_core::{bounded_vec::BoundedVec, U256, ConstU32};
use blockifier::execution::contract_class::ContractClass;
use starknet_client::reader::objects::transaction::{IntermediateInvokeTransaction, IntermediateDeclareTransaction, DeployAccountTransaction, L1HandlerTransaction, DeployTransaction};

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
        Some(Felt252Wrapper::try_from(declare_transaction.class_hash.0.bytes()).unwrap()),
        EntryPointTypeWrapper::External,
        Some(Felt252Wrapper::default()),
        calldata_vec,
        ContractAddressWrapper::default(),
        ContractAddressWrapper::default(),
        Felt252Wrapper::ZERO,
        Some(ClassHashWrapper::ZERO)
    );

    // let version_byte: [u8; 32] = match Felt252Wrapper::try_from(declare_transaction.version.0.into()) {
    //     Ok(valeur) => {
    //         Felt252Wrapper::from(valeur).into()
    //     },
    //     Err(_) => {panic!("Version too long")}
    // };
    // let version_u8: u8 = match version_byte[0] {
    //     0 => 0,
    //     1 => 1,
    //     01 => 2,
    //     _ => panic!("Version not supported")
    // };
    
    Transaction {
        tx_type: TxType::Declare,
        version: Some(u8::default()).unwrap(),
        hash: Felt252Wrapper(declare_transaction.transaction_hash.0.into()),
        signature: signature_vec,
        sender_address: Some(Felt252Wrapper::default()).unwrap(),
        nonce: Felt252Wrapper::try_from(declare_transaction.nonce.0.bytes()).unwrap(),
        call_entrypoint: call_entry_point,
        contract_class: Option::<ContractClass>::default(),
        contract_address_salt: Option::<U256>::default(),
        max_fee: Felt252Wrapper::from(declare_transaction.max_fee.0),
        is_query: false, // Assuming default value
    }
}


pub fn invoke_tx_to_starknet_tx(invoke_transaction : IntermediateInvokeTransaction) -> Transaction {
    let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
    for item in &invoke_transaction.signature.0 {
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
        Some(Felt252Wrapper::default()),   //class_hash: Option<ClassHashWrapper>,
        EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
        Some(Felt252Wrapper::default()),
        calldata_vec,
        ContractAddressWrapper::default(),
        ContractAddressWrapper::default(),
        Felt252Wrapper::ZERO,
        Some(ClassHashWrapper::ZERO)
    );

    let version_invoke = StarkFelt::from(invoke_transaction.version.0);
    let version_byte: [u8; 32] = match Felt252Wrapper::try_from(invoke_transaction.version.0) {
        Ok(valeur) => {
            Felt252Wrapper::from(valeur).into()
        },
        Err(_) => {panic!("Version too long")}
    };
    let version_u8: u8 = match version_byte[0] {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => panic!("Version not supported")
    };

    let sender_address_tx: ContractAddressWrapper =  Felt252Wrapper::from(invoke_transaction.sender_address.0.into());
    
    Transaction {
        tx_type: TxType::Invoke,
        version: Some(u8::default()).unwrap(),
        hash: Felt252Wrapper(invoke_transaction.transaction_hash.0.into()),
        signature: signature_vec,
        sender_address: sender_address_tx,
        nonce: Felt252Wrapper::default(),//Felt252Wrapper::try_from(invoke_transaction.nonce.0.bytes()).unwrap(),
        call_entrypoint: call_entry_point,
        contract_class: Option::<ContractClass>::default(),
        contract_address_salt: Option::<U256>::default(),
        max_fee: Felt252Wrapper::from(invoke_transaction.max_fee.0),
        is_query: false, // Assuming default value
    }
}

// pub fn deploy_tx_to_starknet_tx(mut deploy_transaction : DeployTransaction) -> Transaction {
//     let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
//     for item in &deploy_transaction.signature.0 {
//         match signature_vec.try_push(Felt252Wrapper::try_from(item.bytes()).unwrap()) {
//             Ok(_) => {},
//             Err(_) => {
//                 panic!("Signature too long");
//             }
//         }
//         signature_vec.try_push(Felt252Wrapper::try_from(item.bytes()).unwrap());
//     }
//     let calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();

//     let call_entry_point = CallEntryPointWrapper::new(
//         Some(Felt252Wrapper::try_from(deploy_transaction.class_hash.0.bytes()).unwrap()),   //class_hash: Option<ClassHashWrapper>,
//         EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
//         Some(Felt252Wrapper::default()),
//         calldata_vec,
//         ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
//         ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
//         Felt252Wrapper::ZERO,
//         Some(ClassHashWrapper::ZERO)
//     );
    
//     Transaction {
//         tx_type: TxType::Deploy,
//         version: b'1',
//         hash: Felt252Wrapper(deploy_transaction.transaction_hash.0.into()),
//         signature: signature_vec,
//         sender_address: Some(Felt252Wrapper::default()).unwrap(),
//         nonce: Some(Felt252Wrapper::default()).unwrap(),
//         call_entrypoint: call_entry_point,
//         contract_class: Option::<ContractClass>::default(),
//         contract_address_salt: Option::<U256>::default(),
//         max_fee: Some(Felt252Wrapper::default()).unwrap(),
//         is_query: false, // Assuming default value
//     }
// }

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
        tx_type: TxType::DeployAccount,
        version: b'1',
        hash: Felt252Wrapper(deploy_account_transaction.transaction_hash.0.into()),
        signature: signature_vec,
        sender_address: Some(Felt252Wrapper::default()).unwrap(),
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
    // for item in &l1handler_transaction.signature.0 {
    //     match signature_vec.try_push(Felt252Wrapper::try_from(item.bytes()).unwrap()) {
    //         Ok(_) => {},
    //         Err(_) => {
    //             panic!("Signature too long");
    //         }
    //     }
    //     signature_vec.try_push(Felt252Wrapper::try_from(item.bytes()).unwrap());
    // }
    let calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();

    let call_entry_point = CallEntryPointWrapper::new(
        Some(Felt252Wrapper::default()),   //class_hash: Option<ClassHashWrapper>,
        EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
        Some(Felt252Wrapper::default()),
        calldata_vec,
        ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
        ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
        Felt252Wrapper::ZERO,
        Some(ClassHashWrapper::ZERO)
    );
    
    Transaction {
        tx_type: TxType::L1Handler,
        version: b'1',
        hash: Felt252Wrapper(l1handler_transaction.transaction_hash.0.into()),
        signature: signature_vec,
        sender_address: Some(Felt252Wrapper::default()).unwrap(),
        nonce: Felt252Wrapper::try_from(l1handler_transaction.nonce.0.bytes()).unwrap(),
        call_entrypoint: call_entry_point,
        contract_class: Option::<ContractClass>::default(),
        contract_address_salt: Option::<U256>::default(),
        max_fee: Some(Felt252Wrapper::default()).unwrap(),
        is_query: false, // Assuming default value
    }
}
