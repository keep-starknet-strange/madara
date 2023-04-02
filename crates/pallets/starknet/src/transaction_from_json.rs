use core::str::FromStr;

use blockifier::test_utils::{get_contract_class, ACCOUNT_CONTRACT_PATH};
use frame_support::{bounded_vec, BoundedVec};
use hex::FromHex;
use mp_starknet::execution::{
    CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper, EntryPointTypeWrapper,
};
use mp_starknet::transaction::types::{EventWrapper, MaxArraySize, Transaction};
use sp_core::{H256, U256};

// Deserialization and Conversion for JSON Transactions, Events, and CallEntryPoints
/// Struct for deserializing CallEntryPoint from JSON
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DeserializeCallEntrypoint {
    pub class_hash: Option<String>,
    pub entrypoint_type: String,
    pub entrypoint_selector: Option<String>,
    pub calldata: Vec<String>,
    pub storage_address: String,
    pub caller_address: String,
}

/// Struct for deserializing Event from JSON
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DeserializeEventWrapper {
    pub keys: Vec<String>,
    pub data: Vec<String>,
    pub from_address: String,
}

/// Struct for deserializing Transaction from JSON
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DeserializeTransaction {
    pub version: u8,
    pub hash: String,
    pub signature: Vec<String>,
    pub events: Vec<DeserializeEventWrapper>,
    pub sender_address: String,
    pub nonce: u64,
    pub call_entrypoint: DeserializeCallEntrypoint,
}

// Implement TryFrom trait to convert DeserializeTransaction to Transaction
impl TryFrom<DeserializeTransaction> for Transaction {
    type Error = String;

    fn try_from(d: DeserializeTransaction) -> Result<Self, Self::Error> {
        let version = U256::from(d.version);
        let hash = H256::from_str(&d.hash.as_str()).map_err(|_| "Invalid hash")?;
        let signature = d
            .signature
            .into_iter()
            .map(|s| H256::from_str(&s).map_err(|_| "Invalid signature"))
            .collect::<Result<Vec<H256>, &str>>()?;
        let signature =
            BoundedVec::<H256, MaxArraySize>::try_from(signature).map_err(|_| "Signature exceeds maximum size")?;
        let events = d
            .events
            .into_iter()
            .map(EventWrapper::try_from)
            .collect::<Result<Vec<EventWrapper>, String>>()
            .map_err(|_| "Invalid events")?;
        let events =
            BoundedVec::<EventWrapper, MaxArraySize>::try_from(events).map_err(|_| "Events exceed maximum size")?;
        let sender_address =
            ContractAddressWrapper::from_hex(&d.sender_address).map_err(|_| "Invalid sender address")?;
        let nonce = U256::from(d.nonce);
        let call_entrypoint = CallEntryPointWrapper::try_from(d.call_entrypoint)?;

        Ok(Self { version, hash, signature, events, sender_address, nonce, call_entrypoint, ..Transaction::default() })
    }
}

/// Implement TryFrom trait to convert DeserializeCallEntrypoint to CallEntryPointWrapper
impl TryFrom<DeserializeCallEntrypoint> for CallEntryPointWrapper {
    type Error = String;

    fn try_from(d: DeserializeCallEntrypoint) -> Result<Self, Self::Error> {
        let class_hash = match d.class_hash {
            Some(hash) => Some(<[u8; 32]>::from_hex(&hash).map_err(|_| "Invalid class_hash")?),
            None => None,
        };

        let entrypoint_type = match d.entrypoint_type.as_str() {
            "Constructor" => EntryPointTypeWrapper::Constructor,
            "External" => EntryPointTypeWrapper::External,
            "L1Handler" => EntryPointTypeWrapper::L1Handler,
            _ => return Err("Invalid entrypoint_type".to_string()),
        };

        let entrypoint_selector = match d.entrypoint_selector {
            Some(selector) => Some(H256::from_str(&selector).map_err(|_| "Invalid entrypoint_selector")?),
            None => None,
        };

        let calldata: Result<Vec<H256>, &str> =
            d.calldata.into_iter().map(|hex_str| H256::from_str(&hex_str).map_err(|_| "Invalid calldata")).collect();
        let calldata = BoundedVec::<H256, MaxArraySize>::try_from(calldata?).map_err(|_| "Exceeded max array size")?;

        let storage_address = <[u8; 32]>::from_hex(&d.storage_address).map_err(|_| "Invalid storage_address")?;

        let caller_address = <[u8; 32]>::from_hex(&d.caller_address).map_err(|_| "Invalid caller_address")?;

        Ok(Self { class_hash, entrypoint_type, entrypoint_selector, calldata, storage_address, caller_address })
    }
}

// Implement TryFrom trait to convert DeserializeEventWrapper to EventWrapper
impl TryFrom<DeserializeEventWrapper> for EventWrapper {
    type Error = String;

    fn try_from(d: DeserializeEventWrapper) -> Result<Self, Self::Error> {
        let keys: Result<Vec<H256>, &str> =
            d.keys.into_iter().map(|s| H256::from_str(&s).map_err(|_| "Invalid key")).collect();
        let keys = BoundedVec::<H256, MaxArraySize>::try_from(keys?).map_err(|_| "Exceeded max array size")?;

        let data: Result<Vec<H256>, &str> =
            d.data.into_iter().map(|s| H256::from_str(s.as_str()).map_err(|_| "Invalid data")).collect();
        let data = BoundedVec::<H256, MaxArraySize>::try_from(data?).map_err(|_| "Exceeded max array size")?;

        let from_address =
            H256::from_str(&d.from_address.as_str()).map_err(|_| "Invalid caller_address")?.to_fixed_bytes();

        Ok(Self { keys, data, from_address })
    }
}

/// Create a `Transaction` from a JSON string and an optional contract content.
///
/// This function takes a JSON string (`json_str`) representing a deserialized transaction
/// and a byte slice (`contract_content`) containing the contract content, if available.
///
/// If `contract_content` is not empty, the function will use it to set the `contract_class`
/// field of the resulting `Transaction` object. Otherwise, the `contract_class` field will be set
/// to `None`.
pub fn transaction_from_json(json_str: &str, contract_content: &'static [u8]) -> Result<Transaction, String> {
    let deserialized_transaction: DeserializeTransaction = serde_json::from_str(json_str).map_err(|e| {
        let error_message = format!("Failed to convert deserialized transaction: {:?}", e);
        println!("{}", error_message);
        error_message
    })?;
    let mut transaction = Transaction::try_from(deserialized_transaction).map_err(|e| {
        let error_message = format!("Failed to convert deserialized transaction: {:?}", e);
        println!("{}", error_message);
        error_message
    })?;

    if !contract_content.is_empty() {
        transaction.contract_class = Some(ContractClassWrapper::from(get_contract_class(contract_content)));
    } else {
        transaction.contract_class = None;
    }

    Ok(transaction)
}

#[test]
fn default_transaction() {
    let transaction = Transaction::default();

    let json_content: &str = include_str!("../../../../ressources/transactions/default.json");
    let transaction_from_json =
        transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

    pretty_assertions::assert_eq!(transaction, transaction_from_json);
}

#[test]
fn given_hardcoded_contract_run_deploy_account_tx_then_it_works() {
    let json_content: &str = include_str!(
        "../../../../ressources/transactions/given_hardcoded_contract_run_deploy_account_tx_then_it_works.json"
    );
    let transaction_from_json =
        transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

    let contract_address_str = "02356b628D108863BAf8644c125d97bAD70190AF5957031f4852d00D0F690a77";
    let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

    let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
    let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

    // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
    let transaction = Transaction::new(
        U256::from(1),
        H256::default(),
        bounded_vec!(),
        bounded_vec!(),
        contract_address_bytes,
        U256::from(0),
        CallEntryPointWrapper::new(
            Some(class_hash_bytes),
            EntryPointTypeWrapper::External,
            None,
            bounded_vec![
                // Constructor calldata
            ],
            contract_address_bytes,
            contract_address_bytes,
        ),
        None,
    );
    pretty_assertions::assert_eq!(transaction, transaction_from_json);
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_event_is_emitted() {
    let json_content: &str = include_str!(
        "../../../../ressources/transactions/given_hardcoded_contract_run_invoke_tx_then_event_is_emitted.json"
    );
    let transaction_from_json =
        transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

    let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
    let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

    let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
    let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

    // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
    let transaction = Transaction::new(
        U256::from(1),
        H256::from_str("0x06fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212").unwrap(),
        bounded_vec![
            H256::from_str("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
            H256::from_str("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap()
        ],
        bounded_vec!(),
        contract_address_bytes,
        U256::from(0),
        CallEntryPointWrapper::new(
            Some(class_hash_bytes),
            EntryPointTypeWrapper::External,
            None,
            bounded_vec![
                H256::from_str("0x0624EBFb99865079bd58CFCFB925B6F5Ce940D6F6e41E118b8A72B7163fB435c").unwrap(), /* Contract address */
                H256::from_str("0x00966af5d72d3975f70858b044c77785d3710638bbcebbd33cc7001a91025588").unwrap(), /* Selector */
                H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(), /* Length
                                                                                                                * H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(), // Value */
            ],
            contract_address_bytes,
            contract_address_bytes,
        ),
        None,
    );
    pretty_assertions::assert_eq!(transaction, transaction_from_json);
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_it_works() {
    //
    let json_content: &str =
        include_str!("../../../../ressources/transactions/given_hardcoded_contract_run_invoke_tx_then_it_works.json");
    let transaction_from_json =
        transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

    let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
    let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

    let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
    let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

    // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
    let transaction = Transaction::new(
        U256::from(1),
        H256::from_str("0x06fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212").unwrap(),
        bounded_vec![
            H256::from_str("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
            H256::from_str("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap()
        ],
        bounded_vec!(),
        contract_address_bytes,
        U256::from(0),
        CallEntryPointWrapper::new(
            Some(class_hash_bytes),
            EntryPointTypeWrapper::External,
            None,
            bounded_vec![
                H256::from_str("0x0624EBFb99865079bd58CFCFB925B6F5Ce940D6F6e41E118b8A72B7163fB435c").unwrap(), /* Contract address */
                H256::from_str("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), /* Selector */
                H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), /* Length */
                H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(), // Value
            ],
            contract_address_bytes,
            contract_address_bytes,
        ),
        None,
    );
    pretty_assertions::assert_eq!(transaction, transaction_from_json);
}

#[test]
fn given_hardcoded_contract_run_deploy_account_tx_twice_then_it_fails() {
    //
    let json_content: &str = include_str!(
        "../../../../ressources/transactions/given_hardcoded_contract_run_deploy_account_tx_twice_then_it_fails.json"
    );
    let transaction_from_json =
        transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

    let contract_address_str = "02356b628D108863BAf8644c125d97bAD70190AF5957031f4852d00D0F690a77";
    let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

    let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
    let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

    // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
    let transaction = Transaction::new(
        U256::from(1),
        H256::default(),
        bounded_vec!(),
        bounded_vec!(),
        contract_address_bytes,
        U256::from(0),
        CallEntryPointWrapper::new(
            Some(class_hash_bytes),
            EntryPointTypeWrapper::External,
            None,
            bounded_vec![
                // Constructor calldata
            ],
            contract_address_bytes,
            contract_address_bytes,
        ),
        None,
    );
    pretty_assertions::assert_eq!(transaction, transaction_from_json);
}

#[test]
fn given_hardcoded_contract_run_deploy_account_tx_undeclared_then_it_fails() {
    //
    let json_content: &str = include_str!(
        "../../../../ressources/transactions/given_hardcoded_contract_run_deploy_account_tx_undeclared_then_it_fails.\
         json"
    );
    let transaction_from_json =
        transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

    let contract_address_str = "02356b628D108863BAf8644c125d97bAD70190AF5957031f4852d00D0F690a77";
    let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

    let class_hash_str = "0334e1e4d148a789fb44367eff869a6330693037983ba6fd2291b2be1249e15a";
    let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

    // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
    let transaction = Transaction::new(
        U256::from(1),
        H256::default(),
        bounded_vec!(),
        bounded_vec!(),
        contract_address_bytes,
        U256::from(0),
        CallEntryPointWrapper::new(
            Some(class_hash_bytes),
            EntryPointTypeWrapper::External,
            None,
            bounded_vec![
                // Constructor calldata
            ],
            contract_address_bytes,
            contract_address_bytes,
        ),
        None,
    );
    pretty_assertions::assert_eq!(transaction, transaction_from_json);
}

// This one test also the contract_class
// Weird result, sometimes it fails, sometimes it succeed
// #[test]
// fn given_hardcoded_contract_run_declare_tx_then_it_works() {
// let json_content: &str =
// include_str!("../../../../ressources/transactions/
// given_hardcoded_contract_run_declare_tx_then_it_works.json"); let transaction_from_json =
// transaction_from_json(json_content, ACCOUNT_CONTRACT_PATH).expect("Failed to create Transaction
// from JSON");
//
// let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
// let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();
//
// let class_hash_str = "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
// let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();
//
// let account_class = ContractClassWrapper::from(get_contract_class(ACCOUNT_CONTRACT_PATH));
//
// Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
// let transaction = Transaction::new(
// U256::from(1),
// H256::default(),
// bounded_vec!(),
// bounded_vec!(),
// contract_address_bytes,
// U256::from(0),
// CallEntryPointWrapper::new(
// Some(class_hash_bytes),
// EntryPointTypeWrapper::External,
// None,
// bounded_vec![],
// contract_address_bytes,
// contract_address_bytes,
// ),
// Some(account_class.clone()),
// );
//
// pretty_assertions::assert_eq!(transaction, transaction_from_json);
// }
//
// #[test]
// fn given_hardcoded_contract_run_declare_twice_then_it_fails() {
// let json_content: &str = include_str!(
// "../../../../ressources/transactions/given_hardcoded_contract_run_declare_twice_then_it_fails.
// json" );
// let transaction_from_json =
// transaction_from_json(json_content, ACCOUNT_CONTRACT_PATH).expect("Failed to create Transaction
// from JSON");
//
// let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
// let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();
//
// let class_hash_str = "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
// let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();
//
// let account_class = ContractClassWrapper::from(get_contract_class(ACCOUNT_CONTRACT_PATH));
//
// Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
// let transaction = Transaction::new(
// U256::from(1),
// H256::default(),
// bounded_vec!(),
// bounded_vec!(),
// contract_address_bytes,
// U256::from(0),
// CallEntryPointWrapper::new(
// Some(class_hash_bytes),
// EntryPointTypeWrapper::External,
// None,
// bounded_vec![],
// contract_address_bytes,
// contract_address_bytes,
// ),
// Some(account_class.clone()),
// );
//
// pretty_assertions::assert_eq!(transaction, transaction_from_json);
// }

#[test]
fn given_hardcoded_contract_run_declare_none_then_it_fails() {
    let json_content: &str = include_str!(
        "../../../../ressources/transactions/given_hardcoded_contract_run_declare_none_then_it_fails.json"
    );
    let transaction_from_json =
        transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

    let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
    let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

    let class_hash_str = "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
    let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

    // Example tx : https://testnet.starkscan.co/tx/0x6fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212
    let transaction = Transaction::new(
        U256::from(1),
        H256::default(),
        bounded_vec!(),
        bounded_vec!(),
        contract_address_bytes,
        U256::from(0),
        CallEntryPointWrapper::new(
            Some(class_hash_bytes),
            EntryPointTypeWrapper::External,
            None,
            bounded_vec![],
            contract_address_bytes,
            contract_address_bytes,
        ),
        None,
    );

    pretty_assertions::assert_eq!(transaction, transaction_from_json);
}
