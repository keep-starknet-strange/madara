use mp_starknet::{transaction::types::{Transaction, MaxArraySize, TxType}, execution::types::{Felt252Wrapper, EntryPointTypeWrapper, ContractAddressWrapper, CallEntryPointWrapper, MaxCalldataSize, ClassHashWrapper}};
use sp_core::{bounded_vec::BoundedVec, U256};
use blockifier::execution::contract_class::ContractClass;
use starknet_client::reader::objects::transaction::IntermediateDeclareTransaction;

pub fn declare_tx_to_starknet_tx(declare_transaction: IntermediateDeclareTransaction) -> Transaction {

    println!("declare_transaction: {:?}", declare_transaction);

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




// pub fn invoke_tx_to_starknet_tx(invoke_transaction : InvokeTransaction) -> Transaction {
//     match invoke_transaction {
//         InvokeTransaction::V0(invoke_transaction_v0) => {
//             let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
//             for item in invoke_transaction_v0.signature {
//                 match signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
//                     Ok(_) => {},
//                     Err(_) => {
//                         panic!("Signature too long");
//                     }
//                 }
//                 //signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
//             }
//             let mut calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
//             for item in invoke_transaction_v0.calldata {
//                 match calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
//                     Ok(_) => {},
//                     Err(_) => {
//                         panic!("Calldata too long");
//                     }
//                 }
//                 //calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
//             }

//             let call_entry_point = CallEntryPointWrapper::new(
//                 Some(ClassHashWrapper::default()),   //class_hash: Option<ClassHashWrapper>,
//                 EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
//                 Some(Felt252Wrapper::try_from(invoke_transaction_v0.entry_point_selector.0.as_be_bytes()).unwrap()),
//                 calldata_vec,
//                 ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
//                 ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
// 				Felt252Wrapper::ZERO,
// 				Some(ClassHashWrapper::ZERO)
//             );
//             let tx = Transaction {
//                 tx_type: TxType::Invoke,
//                 version: 0u8,
//                 hash: Felt252Wrapper::try_from(invoke_transaction_v0.transaction_hash.0.as_be_bytes()).unwrap(),
//                 signature: signature_vec,
//                 sender_address: Felt252Wrapper::try_from(invoke_transaction_v0.sender_address.get().as_be_bytes()).unwrap(),
//                 nonce: ContractAddressWrapper::default(),
//                 call_entrypoint: call_entry_point,
//                 contract_class: Option::<ContractClass>::default(),
//                 contract_address_salt: Option::<U256>::default(),
//                 max_fee: Felt252Wrapper::try_from(invoke_transaction_v0.max_fee.0.as_be_bytes()).unwrap(),
//                 ..Transaction::default()
//             };
//             tx
//         }
//         InvokeTransaction::V1(invoke_transaction_v1) => {
//             let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
//             for item in invoke_transaction_v1.signature {
//                 match signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
//                     Ok(_) => {},
//                     Err(_) => {
//                         panic!("Signature too long");
//                     }
//                 }
//                 //signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
//             }

//             let mut calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
//             for item in invoke_transaction_v1.calldata {
//                 match calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
//                     Ok(_) => {},
//                     Err(_) => {
//                         panic!("Calldata too long");
//                     }
//                 }
//                 //calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
//             }

//             let call_entry_point = CallEntryPointWrapper::new(
//                 Some(ClassHashWrapper::default()),   //class_hash: Option<ClassHashWrapper>,
//                 EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
//                 Some(Felt252Wrapper::default()),
//                 calldata_vec,
//                 ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
//                 ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
// 				Felt252Wrapper::ZERO,
// 				Some(ClassHashWrapper::ZERO)
//             );
//             let tx = Transaction {
//                 tx_type: TxType::Invoke,
//                 version: 0u8,
//                 hash: Felt252Wrapper::try_from(invoke_transaction_v1.transaction_hash.0.as_be_bytes()).unwrap(),
//                 signature: signature_vec,
//                 sender_address: Felt252Wrapper::try_from(invoke_transaction_v1.sender_address.get().as_be_bytes()).unwrap(),
//                 nonce: ContractAddressWrapper::default(),
//                 call_entrypoint: call_entry_point,
//                 contract_class: Option::<ContractClass>::default(),
//                 contract_address_salt: Option::<U256>::default(),
//                 max_fee: Felt252Wrapper::try_from(invoke_transaction_v1.max_fee.0.as_be_bytes()).unwrap(),
//                 ..Transaction::default()
//             };
//             tx
//         }
//     }
// }



// pub fn deploy_account_tx_to_starknet_tx(mut deploy_account_transaction : DeployAccountTransaction) -> Transaction {
//     let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
//     for item in deploy_account_transaction.signature {
//         match signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
//             Ok(_) => {},
//             Err(_) => {
//                 panic!("Signature too long");
//             }
//         }
//         //signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
//     }

//     let mut calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
//     for item in deploy_account_transaction.constructor_calldata {
//         match calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
//             Ok(_) => {},
//             Err(_) => {
//                 panic!("Calldata too long");
//             }
//         }
//         //calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
//     }

//     let call_entry_point = CallEntryPointWrapper::new(
//         Some(Felt252Wrapper::try_from(deploy_account_transaction.class_hash.0.as_be_bytes()).unwrap()),   //class_hash: Option<ClassHashWrapper>,
//         EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
//         Some(Felt252Wrapper::default()),
//         calldata_vec,
//         ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
//         ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
// 		Felt252Wrapper::ZERO,
// 		Some(ClassHashWrapper::ZERO)
//     );

//     let tx: Transaction = Transaction {
//         tx_type: TxType::DeployAccount,
//         version: unsafe {
//             *deploy_account_transaction.version.0.as_mut_ptr()
//         },
//         hash: Felt252Wrapper::try_from(deploy_account_transaction.transaction_hash.0.as_be_bytes()).unwrap(),
//         signature: signature_vec,
//         sender_address: Felt252Wrapper::try_from(deploy_account_transaction.contract_address.get().as_be_bytes()).unwrap(),
//         nonce: ContractAddressWrapper::try_from(deploy_account_transaction.nonce.0.as_be_bytes()).unwrap(),
//         call_entrypoint: call_entry_point,
//         contract_class: Option::<ContractClass>::default(),
//         contract_address_salt: Some(U256::try_from(deploy_account_transaction.contract_address_salt.0.as_be_bytes()).unwrap()),
//         max_fee: Felt252Wrapper::try_from(deploy_account_transaction.max_fee.0.as_be_bytes()).unwrap(),
//         is_query: todo!(),
//     };
//     tx
// }

// pub fn l1handler_tx_to_starknet_tx(mut l1hander_transaction : L1HandlerTransaction) -> Transaction {
//     let mut calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
//     for item in l1hander_transaction.calldata {
//         match calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
//             Ok(_) => {},
//             Err(_) => {
//                 panic!("Calldata too long");
//             }
//         }
//         //calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
//     }

//     let call_entry_point = CallEntryPointWrapper::new(
//         Some(ClassHashWrapper::default()),   //class_hash: Option<ClassHashWrapper>,
//         EntryPointTypeWrapper::L1Handler, //entrypoint_type: EntryPointTypeWrapper,
//         Some(Felt252Wrapper::try_from(l1hander_transaction.entry_point_selector.0.as_be_bytes()).unwrap()),
//         calldata_vec,
//         ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
//         ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
// 		Felt252Wrapper::ZERO,
// 		Some(ClassHashWrapper::ZERO)
//     );

//     let tx = Transaction {
//         tx_type: TxType::L1Handler,
//         version: unsafe {
//             *l1hander_transaction.version.0.as_mut_ptr()
//         },
//         hash: Felt252Wrapper::try_from(l1hander_transaction.transaction_hash.0.as_be_bytes()).unwrap(),
//         signature: BoundedVec::default(),
//         sender_address: Felt252Wrapper::try_from(l1hander_transaction.contract_address.get().as_be_bytes()).unwrap(),
//         nonce: ContractAddressWrapper::try_from(l1hander_transaction.nonce.0.as_be_bytes()).unwrap(),
//         call_entrypoint: call_entry_point,
//         contract_class: Option::<ContractClass>::default(),
//         contract_address_salt: Option::<U256>::default(),
//         max_fee: Felt252Wrapper::ONE,
//         is_query: todo!(),
//     };
//     tx
// }
