use std::sync::Arc;

use blockifier::abi::abi_utils::get_fee_token_var_address;
use blockifier::abi::sierra_types::next_storage_key;
use blockifier::execution::contract_class::ClassInfo;
use blockifier::state::state_api::State;
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use starknet_api::core::{
    calculate_contract_address, ClassHash, ContractAddress, EntryPointSelector, Nonce, PatriciaKey,
};
use starknet_api::data_availability::DataAvailabilityMode;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{
    Calldata, ContractAddressSalt, DeclareTransactionV0V1, DeployAccountTransactionV1, Fee, TransactionSignature,
    TransactionVersion,
};

use self::mock::default_mock::Starknet;
use self::mock::{get_account_address, AccountType};
use self::utils::{create_resource_bounds, get_contract_class};
use crate::blockifier_state_adapter::BlockifierStateAdapter;
use crate::tests::mock::account_helper;
use crate::tests::utils::sign_message_hash;
use crate::{Config, Nonces};

mod account_helper;
mod build_genesis_config;
mod call_contract;
mod declare_tx;
mod deploy_account_tx;
mod erc20;
mod events;
mod fees_disabled;
mod genesis_block;
mod invoke_tx;
mod l1_handler_validation;
mod l1_message;
mod query_tx;
mod re_execute_transactions;
mod send_message;
mod sequencer_address;

mod block;
mod constants;
mod mock;
mod utils;

const MAX_FEE: Fee = Fee(u64::MAX as u128);

// ref: https://github.com/tdelabro/blockifier/blob/no_std-support/crates/blockifier/feature_contracts/account_without_validations.cairo
pub fn get_invoke_dummy(
    chain_id: Felt252Wrapper,
    nonce: Nonce,
) -> blockifier::transaction::transactions::InvokeTransaction {
    let signature = TransactionSignature(vec![
        StarkFelt::try_from("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        StarkFelt::try_from("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    ]);
    let sender_address =
        ContractAddress(PatriciaKey(StarkFelt::try_from(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap()));
    let calldata = Calldata(Arc::new(vec![
        StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), /* contract_address */
        StarkFelt::try_from("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), /* selector for the `with_arg` external */
        StarkFelt::try_from("0x1").unwrap(),  // calldata_len
        StarkFelt::try_from("0x19").unwrap(), // calldata[0]
    ]));

    let tx = starknet_api::transaction::InvokeTransaction::V1(starknet_api::transaction::InvokeTransactionV1 {
        max_fee: MAX_FEE,
        signature,
        nonce,
        sender_address,
        calldata,
    });

    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::InvokeTransaction { tx, tx_hash, only_query: false }
}

pub fn get_invoke_v3_dummy(
    chain_id: Felt252Wrapper,
    nonce: Nonce,
) -> blockifier::transaction::transactions::InvokeTransaction {
    let signature = TransactionSignature(vec![
        StarkFelt::try_from("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        StarkFelt::try_from("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    ]);
    let sender_address =
        ContractAddress(PatriciaKey(StarkFelt::try_from(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap()));
    let calldata = Calldata(Arc::new(vec![
        StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), /* contract_address */
        StarkFelt::try_from("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), /* selector for the `with_arg` external */
        StarkFelt::try_from("0x1").unwrap(),  // calldata_len
        StarkFelt::try_from("0x19").unwrap(), // calldata[0]
    ]));

    let tx = starknet_api::transaction::InvokeTransaction::V3(starknet_api::transaction::InvokeTransactionV3 {
        resource_bounds: create_resource_bounds(),
        tip: starknet_api::transaction::Tip::default(),
        calldata,
        sender_address,
        nonce,
        signature,
        nonce_data_availability_mode: DataAvailabilityMode::L1,
        fee_data_availability_mode: DataAvailabilityMode::L1,
        paymaster_data: starknet_api::transaction::PaymasterData(vec![StarkFelt::ZERO]),
        account_deployment_data: starknet_api::transaction::AccountDeploymentData(vec![StarkFelt::ZERO]),
    });

    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::InvokeTransaction { tx, tx_hash, only_query: false }
}

// ref: https://github.com/argentlabs/argent-contracts-starknet/blob/develop/contracts/account/ArgentAccount.cairo
fn get_invoke_argent_dummy(chain_id: Felt252Wrapper) -> blockifier::transaction::transactions::InvokeTransaction {
    let sender_address = ContractAddress(PatriciaKey(
        StarkFelt::try_from("0x02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5").unwrap(),
    ));
    let nonce = Nonce(StarkFelt::ZERO);
    let signature = TransactionSignature::default();
    let calldata = Calldata(Arc::new(vec![
        StarkFelt::try_from("0x1").unwrap(), // call_array_len
        StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), // to
        StarkFelt::try_from("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), // selector
        StarkFelt::try_from("0x0").unwrap(), // data_offset
        StarkFelt::try_from("0x1").unwrap(), // data_len
        StarkFelt::try_from("0x1").unwrap(), // calldata_len
        StarkFelt::try_from("0x19").unwrap(), // calldata[0]
    ]));

    let tx = starknet_api::transaction::InvokeTransaction::V1(starknet_api::transaction::InvokeTransactionV1 {
        max_fee: MAX_FEE,
        signature,
        nonce,
        sender_address,
        calldata,
    });

    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::InvokeTransaction { tx, tx_hash, only_query: false }
}

// ref: https://github.com/myBraavos/braavos-account-cairo/blob/develop/src/account/Account.cairo
fn get_invoke_braavos_dummy(chain_id: Felt252Wrapper) -> blockifier::transaction::transactions::InvokeTransaction {
    let signature = TransactionSignature(vec![
        StarkFelt::try_from("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        StarkFelt::try_from("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    ]);
    let sender_address = ContractAddress(PatriciaKey(
        StarkFelt::try_from("0x05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122").unwrap(),
    ));
    let nonce = Nonce(StarkFelt::ZERO);
    let calldata = Calldata(Arc::new(vec![
        StarkFelt::try_from("0x1").unwrap(), // call_array_len
        StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), // to
        StarkFelt::try_from("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), // selector
        StarkFelt::try_from("0x0").unwrap(), // data_offset
        StarkFelt::try_from("0x1").unwrap(), // data_len
        StarkFelt::try_from("0x1").unwrap(), // calldata_len
        StarkFelt::try_from("0x19").unwrap(), // calldata[0]
    ]));

    let tx = starknet_api::transaction::InvokeTransaction::V1(starknet_api::transaction::InvokeTransactionV1 {
        max_fee: MAX_FEE,
        signature,
        nonce,
        sender_address,
        calldata,
    });

    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::InvokeTransaction { tx, tx_hash, only_query: false }
}

// ref: https://github.com/OpenZeppelin/cairo-contracts/blob/main/src/openzeppelin/token/erc20/IERC20.cairo
fn get_invoke_emit_event_dummy(chain_id: Felt252Wrapper) -> blockifier::transaction::transactions::InvokeTransaction {
    let signature = TransactionSignature(vec![
        StarkFelt::try_from("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        StarkFelt::try_from("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    ]);
    let sender_address = ContractAddress(PatriciaKey(
        StarkFelt::try_from("0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0").unwrap(),
    ));
    let nonce = Nonce(StarkFelt::ZERO);
    let calldata = Calldata(Arc::new(vec![
        StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), // to
        StarkFelt::try_from("0x00966af5d72d3975f70858b044c77785d3710638bbcebbd33cc7001a91025588").unwrap(), /* selector */
        StarkFelt::try_from("0x0").unwrap(),                                                                /* amount */
    ]));

    let tx = starknet_api::transaction::InvokeTransaction::V1(starknet_api::transaction::InvokeTransactionV1 {
        max_fee: MAX_FEE,
        signature,
        nonce,
        sender_address,
        calldata,
    });

    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::InvokeTransaction { tx, tx_hash, only_query: false }
}

// ref: https://github.com/tdelabro/blockifier/blob/no_std-support/crates/blockifier/feature_contracts/account_without_validations.cairo
fn get_invoke_nonce_dummy(chain_id: Felt252Wrapper) -> blockifier::transaction::transactions::InvokeTransaction {
    let signature = TransactionSignature(vec![
        StarkFelt::try_from("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        StarkFelt::try_from("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    ]);
    let sender_address =
        ContractAddress(PatriciaKey(StarkFelt::try_from(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap()));
    let nonce = Nonce(StarkFelt::ONE);
    let calldata = Calldata(Arc::new(vec![
        StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), /* contract_address */
        StarkFelt::try_from("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), /* selector */
        StarkFelt::try_from("0x1").unwrap(),  // calldata_len
        StarkFelt::try_from("0x19").unwrap(), // calldata[0]
    ]));

    let tx = starknet_api::transaction::InvokeTransaction::V1(starknet_api::transaction::InvokeTransactionV1 {
        max_fee: MAX_FEE,
        signature,
        nonce,
        sender_address,
        calldata,
    });

    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::InvokeTransaction { tx, tx_hash, only_query: false }
}

// ref: https://github.com/keep-starknet-strange/madara/blob/main/cairo-contracts/src/accounts/NoValidateAccount.cairo
fn get_storage_read_write_dummy(chain_id: Felt252Wrapper) -> blockifier::transaction::transactions::InvokeTransaction {
    let signature = TransactionSignature::default();
    let sender_address =
        ContractAddress(PatriciaKey(StarkFelt::try_from(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap()));
    let nonce = Nonce(StarkFelt::ZERO);
    let calldata = Calldata(Arc::new(vec![
        StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), /* contract_address */
        StarkFelt::try_from("0x03b097c62d3e4b85742aadd0dfb823f96134b886ec13bda57b68faf86f294d97").unwrap(), /* selector */
        StarkFelt::try_from("0x2").unwrap(),  // calldata_len
        StarkFelt::try_from("0x19").unwrap(), // calldata[0]
        StarkFelt::try_from("0x1").unwrap(),  // calldata[1]
    ]));

    let tx = starknet_api::transaction::InvokeTransaction::V1(starknet_api::transaction::InvokeTransactionV1 {
        max_fee: MAX_FEE,
        signature,
        nonce,
        sender_address,
        calldata,
    });

    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::InvokeTransaction { tx, tx_hash, only_query: false }
}

// ref: https://github.com/OpenZeppelin/cairo-contracts/blob/main/src/openzeppelin/account/IAccount.cairo
fn get_invoke_openzeppelin_dummy(chain_id: Felt252Wrapper) -> blockifier::transaction::transactions::InvokeTransaction {
    let signature = TransactionSignature(vec![
        StarkFelt::try_from("0x028ef1ae6c37314bf9df65663db1cf68f95d67c4b4cf7f6590654933a84912b0").unwrap(),
        StarkFelt::try_from("0x0625aae99c58b18e5161c719fef0f99579c6468ca6c1c866f9b2b968a5447e4").unwrap(),
    ]);
    let sender_address = ContractAddress(PatriciaKey(
        StarkFelt::try_from("0x06e2616a2dceff4355997369246c25a78e95093df7a49e5ca6a06ce1544ffd50").unwrap(),
    ));
    let nonce = Nonce(StarkFelt::ZERO);
    let calldata = Calldata(Arc::new(vec![
        StarkFelt::try_from("0x1").unwrap(), // call_array_len
        StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), // to
        StarkFelt::try_from("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), // selector
        StarkFelt::try_from("0x0").unwrap(), // data offset
        StarkFelt::try_from("0x1").unwrap(), // data length
        StarkFelt::try_from("0x1").unwrap(), // calldata_len
        StarkFelt::try_from("0x19").unwrap(), // calldata[0]
    ]));

    let tx = starknet_api::transaction::InvokeTransaction::V1(starknet_api::transaction::InvokeTransactionV1 {
        max_fee: MAX_FEE,
        signature,
        nonce,
        sender_address,
        calldata,
    });

    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::InvokeTransaction { tx, tx_hash, only_query: false }
}

/// Returns a dummy declare transaction for the given account type.
/// The declared class hash is a ERC20 contract, class hash calculated
/// with starkli.
pub fn get_declare_dummy(
    chain_id: Felt252Wrapper,
    nonce: Nonce,
    account_type: AccountType,
) -> blockifier::transaction::transactions::DeclareTransaction {
    let account_addr = get_account_address(None, account_type);

    let erc20_class_hash =
        ClassHash(StarkFelt::try_from("0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4").unwrap());
    let erc20_class = get_contract_class("ERC20.json", 0);

    let mut tx = starknet_api::transaction::DeclareTransaction::V1(DeclareTransactionV0V1 {
        max_fee: MAX_FEE,
        signature: TransactionSignature(vec![]),
        nonce,
        class_hash: erc20_class_hash,
        sender_address: account_addr,
    });

    let tx_hash = tx.compute_hash(chain_id, false);
    let signature = sign_message_hash(tx_hash);

    if let starknet_api::transaction::DeclareTransaction::V1(tx) = &mut tx {
        tx.signature = signature;
    }

    let class_info = ClassInfo::new(&erc20_class, 0, 1).unwrap();

    blockifier::transaction::transactions::DeclareTransaction::new(tx, tx_hash, class_info).unwrap()
}

/// Returns a dummy deploy account transaction for the given salt and account type
pub fn get_deploy_account_dummy(
    chain_id: Felt252Wrapper,
    nonce: Nonce,
    contract_address_salt: ContractAddressSalt,
    account_type: AccountType,
) -> blockifier::transaction::transactions::DeployAccountTransaction {
    let (account_class_hash, calldata) = account_helper(account_type);

    let tx = starknet_api::transaction::DeployAccountTransaction::V1(DeployAccountTransactionV1 {
        max_fee: Fee(u64::MAX as u128),
        signature: TransactionSignature(vec![]),
        nonce,
        contract_address_salt,
        constructor_calldata: calldata,
        class_hash: account_class_hash,
    });
    let tx_hash = tx.compute_hash(chain_id, false);
    let contract_address = calculate_contract_address(
        tx.contract_address_salt(),
        tx.class_hash(),
        &tx.constructor_calldata(),
        Default::default(),
    )
    .unwrap();

    blockifier::transaction::transactions::DeployAccountTransaction { tx, tx_hash, contract_address, only_query: false }
}

pub fn create_l1_handler_transaction(
    chain_id: Felt252Wrapper,
    nonce: Nonce,
    contract_address: Option<ContractAddress>,
    entry_point_selector: Option<EntryPointSelector>,
    calldata: Option<Calldata>,
) -> blockifier::transaction::transactions::L1HandlerTransaction {
    let tx = starknet_api::transaction::L1HandlerTransaction {
        nonce,
        contract_address: contract_address.unwrap_or_default(),
        entry_point_selector: entry_point_selector.unwrap_or_default(),
        calldata: calldata.unwrap_or_default(),
        version: TransactionVersion(StarkFelt::ZERO),
    };

    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::L1HandlerTransaction { tx, tx_hash, paid_fee_on_l1: Fee(100) }
}

/// Sets the balance of the given address to infinite.
pub fn set_infinite_tokens<T: Config>(contract_address: &ContractAddress) {
    let fee_token_addresses = Starknet::fee_token_addresses();
    let balance_key_low = get_fee_token_var_address(*contract_address);
    let balance_key_high = next_storage_key(&balance_key_low).expect("Cannot get balance high key.");

    let mut state_adapter = BlockifierStateAdapter::<T>::default();

    state_adapter
        .set_storage_at(fee_token_addresses.eth_fee_token_address, balance_key_low, StarkFelt::from(u64::MAX as u128))
        .unwrap();
    state_adapter
        .set_storage_at(fee_token_addresses.eth_fee_token_address, balance_key_high, StarkFelt::from(u64::MAX as u128))
        .unwrap();
    state_adapter
        .set_storage_at(fee_token_addresses.strk_fee_token_address, balance_key_low, StarkFelt::from(u64::MAX as u128))
        .unwrap();
    state_adapter
        .set_storage_at(fee_token_addresses.strk_fee_token_address, balance_key_high, StarkFelt::from(u64::MAX as u128))
        .unwrap();
}

/// Sets nonce for the given address.
pub fn set_nonce<T: Config>(address: &ContractAddress, nonce: &Nonce) {
    Nonces::<T>::insert(address, nonce)
}
