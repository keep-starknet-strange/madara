use mp_starknet::{transaction::types::{Transaction, MaxArraySize, TxType}, execution::types::{Felt252Wrapper, EntryPointTypeWrapper, ContractAddressWrapper, CallEntryPointWrapper, MaxCalldataSize, ClassHashWrapper}};
use sp_core::{bounded_vec::BoundedVec, U256};
use starknet_gateway_types::reply::transaction::{DeclareTransaction, InvokeTransaction, DeployAccountTransaction, L1HandlerTransaction};
use blockifier::execution::contract_class::ContractClass;

pub fn declare_tx_to_starknet_tx(declare_transaction : DeclareTransaction) -> Transaction {
    match declare_transaction {
        DeclareTransaction::V0(declare_transactionv0v1) => {
            let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
            for item in declare_transactionv0v1.signature {
                match signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Signature too long");
                    }
                }
                //signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            }
            let calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
            // for item in declare_transactionv0v1.calldata {
            //     calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            // }

            let call_entry_point = CallEntryPointWrapper::new(
                Some(Felt252Wrapper::try_from(declare_transactionv0v1.class_hash.0.as_be_bytes()).unwrap()),   //class_hash: Option<ClassHashWrapper>,
                EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
                Some(Felt252Wrapper::default()),
                calldata_vec,
                ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
                ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
				Felt252Wrapper::ZERO,
				Some(ClassHashWrapper::ZERO)
            );
            let tx = Transaction {
                tx_type: TxType::Declare,
                version: 0u8,
                hash: Felt252Wrapper::try_from(declare_transactionv0v1.transaction_hash.0.as_be_bytes()).unwrap(),
                signature: signature_vec,
                sender_address: Felt252Wrapper::try_from(declare_transactionv0v1.sender_address.get().as_be_bytes()).unwrap(),
                nonce: ContractAddressWrapper::try_from(declare_transactionv0v1.nonce.0.as_be_bytes()).unwrap(),
                call_entrypoint: call_entry_point,
                contract_class: Option::<ContractClass>::default(),
                contract_address_salt: Option::<U256>::default(),
                max_fee: Felt252Wrapper::try_from(declare_transactionv0v1.max_fee.0.as_be_bytes()).unwrap(),
                ..Transaction::default()
            };
            tx
        }
        DeclareTransaction::V1(declare_transactionv0v1) => {
            let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
            for item in declare_transactionv0v1.signature {
                match signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Signature too long");
                    }
                }
                //signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            }

            let calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
            // for item in declare_transactionv0v1.calldata {
            //     calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            // }

            let call_entry_point = CallEntryPointWrapper::new(
                Some(Felt252Wrapper::try_from(declare_transactionv0v1.class_hash.0.as_be_bytes()).unwrap()),   //class_hash: Option<ClassHashWrapper>,
                EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
                Some(Felt252Wrapper::default()),
                calldata_vec,
                ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
                ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
				Felt252Wrapper::ZERO,
				Some(ClassHashWrapper::ZERO)
            );
            let tx = Transaction {
                tx_type: TxType::Declare,
                version: 0u8,
                hash: Felt252Wrapper::try_from(declare_transactionv0v1.transaction_hash.0.as_be_bytes()).unwrap(),
                signature: signature_vec,
                sender_address: Felt252Wrapper::try_from(declare_transactionv0v1.sender_address.get().as_be_bytes()).unwrap(),
                nonce: ContractAddressWrapper::try_from(declare_transactionv0v1.nonce.0.as_be_bytes()).unwrap(),
                call_entrypoint: call_entry_point,
                contract_class: Option::<ContractClass>::default(),
                contract_address_salt: Option::<U256>::default(),
                max_fee: Felt252Wrapper::try_from(declare_transactionv0v1.max_fee.0.as_be_bytes()).unwrap(),
                ..Transaction::default()
            };
            tx
        }
        DeclareTransaction::V2(declare_transactionv2) => {
            let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
            for item in declare_transactionv2.signature {
                match signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Signature too long");
                    }
                }
                //signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            }

            let calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
            // for item in declare_transactionv2.calldata {
            //     calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            // }

            let call_entry_point = CallEntryPointWrapper::new(
                Some(Felt252Wrapper::try_from(declare_transactionv2.class_hash.0.as_be_bytes()).unwrap()),   //class_hash: Option<ClassHashWrapper>,
                EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
                Some(Felt252Wrapper::default()),
                calldata_vec,
                ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
                ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
				Felt252Wrapper::ZERO,
				Some(ClassHashWrapper::ZERO)
            );
            let tx = Transaction {
                tx_type: TxType::Declare,
                version: 0u8,
                hash: Felt252Wrapper::try_from(declare_transactionv2.transaction_hash.0.as_be_bytes()).unwrap(),
                signature: signature_vec,
                sender_address: Felt252Wrapper::try_from(declare_transactionv2.sender_address.get().as_be_bytes()).unwrap(),
                nonce: ContractAddressWrapper::try_from(declare_transactionv2.nonce.0.as_be_bytes()).unwrap(),
                call_entrypoint: call_entry_point,
                contract_class: Option::<ContractClass>::default(),
                contract_address_salt: Option::<U256>::default(),
                max_fee: Felt252Wrapper::try_from(declare_transactionv2.max_fee.0.as_be_bytes()).unwrap(),
                ..Transaction::default()
            };
            tx
        }
    }
}



pub fn invoke_tx_to_starknet_tx(invoke_transaction : InvokeTransaction) -> Transaction {
    match invoke_transaction {
        InvokeTransaction::V0(invoke_transaction_v0) => {
            let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
            for item in invoke_transaction_v0.signature {
                match signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Signature too long");
                    }
                }
                //signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            }
            let mut calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
            for item in invoke_transaction_v0.calldata {
                match calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Calldata too long");
                    }
                }
                //calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            }

            let call_entry_point = CallEntryPointWrapper::new(
                Some(ClassHashWrapper::default()),   //class_hash: Option<ClassHashWrapper>,
                EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
                Some(Felt252Wrapper::try_from(invoke_transaction_v0.entry_point_selector.0.as_be_bytes()).unwrap()),
                calldata_vec,
                ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
                ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
				Felt252Wrapper::ZERO,
				Some(ClassHashWrapper::ZERO)
            );
            let tx = Transaction {
                tx_type: TxType::Invoke,
                version: 0u8,
                hash: Felt252Wrapper::try_from(invoke_transaction_v0.transaction_hash.0.as_be_bytes()).unwrap(),
                signature: signature_vec,
                sender_address: Felt252Wrapper::try_from(invoke_transaction_v0.sender_address.get().as_be_bytes()).unwrap(),
                nonce: ContractAddressWrapper::default(),
                call_entrypoint: call_entry_point,
                contract_class: Option::<ContractClass>::default(),
                contract_address_salt: Option::<U256>::default(),
                max_fee: Felt252Wrapper::try_from(invoke_transaction_v0.max_fee.0.as_be_bytes()).unwrap(),
                ..Transaction::default()
            };
            tx
        }
        InvokeTransaction::V1(invoke_transaction_v1) => {
            let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
            for item in invoke_transaction_v1.signature {
                match signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Signature too long");
                    }
                }
                //signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            }

            let mut calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
            for item in invoke_transaction_v1.calldata {
                match calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Calldata too long");
                    }
                }
                //calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
            }

            let call_entry_point = CallEntryPointWrapper::new(
                Some(ClassHashWrapper::default()),   //class_hash: Option<ClassHashWrapper>,
                EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
                Some(Felt252Wrapper::default()),
                calldata_vec,
                ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
                ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
				Felt252Wrapper::ZERO,
				Some(ClassHashWrapper::ZERO)
            );
            let tx = Transaction {
                tx_type: TxType::Invoke,
                version: 0u8,
                hash: Felt252Wrapper::try_from(invoke_transaction_v1.transaction_hash.0.as_be_bytes()).unwrap(),
                signature: signature_vec,
                sender_address: Felt252Wrapper::try_from(invoke_transaction_v1.sender_address.get().as_be_bytes()).unwrap(),
                nonce: ContractAddressWrapper::default(),
                call_entrypoint: call_entry_point,
                contract_class: Option::<ContractClass>::default(),
                contract_address_salt: Option::<U256>::default(),
                max_fee: Felt252Wrapper::try_from(invoke_transaction_v1.max_fee.0.as_be_bytes()).unwrap(),
                ..Transaction::default()
            };
            tx
        }
    }
}



pub fn deploy_account_tx_to_starknet_tx(mut deploy_account_transaction : DeployAccountTransaction) -> Transaction {
    let mut signature_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
    for item in deploy_account_transaction.signature {
        match signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
            Ok(_) => {},
            Err(_) => {
                panic!("Signature too long");
            }
        }
        //signature_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
    }

    let mut calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
    for item in deploy_account_transaction.constructor_calldata {
        match calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
            Ok(_) => {},
            Err(_) => {
                panic!("Calldata too long");
            }
        }
        //calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
    }

    let call_entry_point = CallEntryPointWrapper::new(
        Some(Felt252Wrapper::try_from(deploy_account_transaction.class_hash.0.as_be_bytes()).unwrap()),   //class_hash: Option<ClassHashWrapper>,
        EntryPointTypeWrapper::External, //entrypoint_type: EntryPointTypeWrapper,
        Some(Felt252Wrapper::default()),
        calldata_vec,
        ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
        ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
		Felt252Wrapper::ZERO,
		Some(ClassHashWrapper::ZERO)
    );

    let tx: Transaction = Transaction {
        tx_type: TxType::DeployAccount,
        version: unsafe {
            *deploy_account_transaction.version.0.as_mut_ptr()
        },
        hash: Felt252Wrapper::try_from(deploy_account_transaction.transaction_hash.0.as_be_bytes()).unwrap(),
        signature: signature_vec,
        sender_address: Felt252Wrapper::try_from(deploy_account_transaction.contract_address.get().as_be_bytes()).unwrap(),
        nonce: ContractAddressWrapper::try_from(deploy_account_transaction.nonce.0.as_be_bytes()).unwrap(),
        call_entrypoint: call_entry_point,
        contract_class: Option::<ContractClass>::default(),
        contract_address_salt: Some(U256::try_from(deploy_account_transaction.contract_address_salt.0.as_be_bytes()).unwrap()),
        max_fee: Felt252Wrapper::try_from(deploy_account_transaction.max_fee.0.as_be_bytes()).unwrap(),
        is_query: todo!(),
    };
    tx
}

pub fn l1handler_tx_to_starknet_tx(mut l1hander_transaction : L1HandlerTransaction) -> Transaction {
    let mut calldata_vec: BoundedVec<Felt252Wrapper, MaxCalldataSize> = BoundedVec::new();
    for item in l1hander_transaction.calldata {
        match calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap()) {
            Ok(_) => {},
            Err(_) => {
                panic!("Calldata too long");
            }
        }
        //calldata_vec.try_push(Felt252Wrapper::try_from(item.0.as_be_bytes()).unwrap());
    }

    let call_entry_point = CallEntryPointWrapper::new(
        Some(ClassHashWrapper::default()),   //class_hash: Option<ClassHashWrapper>,
        EntryPointTypeWrapper::L1Handler, //entrypoint_type: EntryPointTypeWrapper,
        Some(Felt252Wrapper::try_from(l1hander_transaction.entry_point_selector.0.as_be_bytes()).unwrap()),
        calldata_vec,
        ContractAddressWrapper::default(), //storage_address: ContractAddressWrapper,
        ContractAddressWrapper::default(), //caller_address: ContractAddressWrapper,
		Felt252Wrapper::ZERO,
		Some(ClassHashWrapper::ZERO)
    );

    let tx = Transaction {
        tx_type: TxType::L1Handler,
        version: unsafe {
            *l1hander_transaction.version.0.as_mut_ptr()
        },
        hash: Felt252Wrapper::try_from(l1hander_transaction.transaction_hash.0.as_be_bytes()).unwrap(),
        signature: BoundedVec::default(),
        sender_address: Felt252Wrapper::try_from(l1hander_transaction.contract_address.get().as_be_bytes()).unwrap(),
        nonce: ContractAddressWrapper::try_from(l1hander_transaction.nonce.0.as_be_bytes()).unwrap(),
        call_entrypoint: call_entry_point,
        contract_class: Option::<ContractClass>::default(),
        contract_address_salt: Option::<U256>::default(),
        max_fee: Felt252Wrapper::ONE,
        is_query: todo!(),
    };
    tx
}
