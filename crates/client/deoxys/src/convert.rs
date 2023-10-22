//! Converts types from [`starknet_gateway`] to madara's expected types.

use starknet_api::hash::StarkFelt;
use starknet_ff::FieldElement;
use starknet_gateway::sequencer::models as p;

pub fn block(block: &p::Block) -> mp_block::Block {
    let transactions = transactions(&block.transactions);
    let events = events(&block.transaction_receipts);
    let block_number = block.block_number.expect("no block number provided");
    let sequencer_address = block.sequencer_address
        .map_or(contract_address(FieldElement::ZERO), |addr| contract_address(addr));
    let (transaction_commitment, event_commitment) = commitments(&transactions, &events, block_number);

    let header = mp_block::Header {
        parent_block_hash: felt(block.parent_block_hash),
        block_number,
        status: block_status(&block.status),
        block_timestamp: block.timestamp,
        global_state_root: felt(block.state_root.expect("no state root provided")),
        sequencer_address: sequencer_address,
        transaction_count: block.transactions.len() as u128,
        transaction_commitment,
        event_count: events.len() as u128,
        event_commitment,
        protocol_version: 0,
        extra_data: block.block_hash.map(|h| sp_core::U256::from_big_endian(&h.to_bytes_be())),
    };

    mp_block::Block::new(header, transactions)
}

fn block_status(status: &p::BlockStatus) -> mp_block::BlockStatus {
    match status {
        p::BlockStatus::Aborted => mp_block::BlockStatus::Rejected,
        p::BlockStatus::AcceptedOnL1 => mp_block::BlockStatus::AcceptedOnL1,
        p::BlockStatus::AcceptedOnL2 => mp_block::BlockStatus::AcceptedOnL2,
        p::BlockStatus::Pending => mp_block::BlockStatus::Pending,
        p::BlockStatus::Reverted => panic!("reverted block found"),
    }
}

fn transactions(txs: &[p::TransactionType]) -> Vec<mp_transactions::Transaction> {
    txs.iter().map(transaction).collect()
}

fn transaction(transaction: &p::TransactionType) -> mp_transactions::Transaction {
    match transaction {
        p::TransactionType::InvokeFunction(tx) => mp_transactions::Transaction::Invoke(invoke_transaction(tx)),
        p::TransactionType::Declare(tx) => mp_transactions::Transaction::Declare(declare_transaction(tx)),
        p::TransactionType::Deploy(tx) => mp_transactions::Transaction::Deploy(deploy_transaction(tx)),
        p::TransactionType::DeployAccount(tx) => {
            mp_transactions::Transaction::DeployAccount(deploy_account_transaction(tx))
        }
        p::TransactionType::L1Handler(tx) => mp_transactions::Transaction::L1Handler(l1_handler_transaction(tx)),
    }
}

fn invoke_transaction(tx: &p::InvokeFunctionTransaction) -> mp_transactions::InvokeTransaction {
    if tx.version == FieldElement::ZERO {
        mp_transactions::InvokeTransaction::V0(mp_transactions::InvokeTransactionV0 {
            max_fee: fee(tx.max_fee),
            signature: tx.signature.iter().copied().map(felt).map(Into::into).collect(),
            contract_address: felt(tx.sender_address).into(),
            entry_point_selector: felt(tx.entry_point_selector.expect("no entry_point_selector provided")).into(),
            calldata: tx.calldata.iter().copied().map(felt).map(Into::into).collect(),
        })
    } else {
        mp_transactions::InvokeTransaction::V1(mp_transactions::InvokeTransactionV1 {
            max_fee: fee(tx.max_fee),
            signature: tx.signature.iter().copied().map(felt).map(Into::into).collect(),
            nonce: felt(tx.nonce.expect("no nonce provided")).into(),
            sender_address: felt(tx.sender_address).into(),
            calldata: tx.calldata.iter().copied().map(felt).map(Into::into).collect(),
        })
    }
}

fn declare_transaction(tx: &p::DeclareTransaction) -> mp_transactions::DeclareTransaction {
    if tx.version == FieldElement::ZERO {
        mp_transactions::DeclareTransaction::V0(mp_transactions::DeclareTransactionV0 {
            max_fee: fee(tx.max_fee),
            signature: tx.signature.iter().copied().map(felt).map(Into::into).collect(),
            nonce: felt(tx.nonce).into(),
            class_hash: felt(tx.class_hash).into(),
            sender_address: felt(tx.sender_address).into(),
        })
    } else if tx.version == FieldElement::ONE {
        mp_transactions::DeclareTransaction::V1(mp_transactions::DeclareTransactionV1 {
            max_fee: fee(tx.max_fee),
            signature: tx.signature.iter().copied().map(felt).map(Into::into).collect(),
            nonce: felt(tx.nonce).into(),
            class_hash: felt(tx.class_hash).into(),
            sender_address: felt(tx.sender_address).into(),
        })
    } else {
        mp_transactions::DeclareTransaction::V2(mp_transactions::DeclareTransactionV2 {
            max_fee: fee(tx.max_fee),
            signature: tx.signature.iter().copied().map(felt).map(Into::into).collect(),
            nonce: felt(tx.nonce).into(),
            class_hash: felt(tx.class_hash).into(),
            sender_address: felt(tx.sender_address).into(),
            compiled_class_hash: felt(tx.compiled_class_hash.expect("no class hash available")).into(),
        })
    }
}

fn deploy_transaction(tx: &p::DeployTransaction) -> mp_transactions::DeployTransaction {
    mp_transactions::DeployTransaction {
        version: starknet_api::transaction::TransactionVersion(felt(tx.version)),
        class_hash: felt(tx.class_hash).into(),
        contract_address_salt: felt(tx.contract_address_salt).into(),
        constructor_calldata: tx.constructor_calldata.iter().copied().map(felt).map(Into::into).collect(),
    }
}

fn deploy_account_transaction(tx: &p::DeployAccountTransaction) -> mp_transactions::DeployAccountTransaction {
    mp_transactions::DeployAccountTransaction {
        max_fee: fee(tx.max_fee),
        signature: tx.signature.iter().copied().map(felt).map(Into::into).collect(),
        nonce: felt(tx.nonce).into(),
        contract_address_salt: felt(tx.contract_address_salt).into(),
        constructor_calldata: tx.constructor_calldata.iter().copied().map(felt).map(Into::into).collect(),
        class_hash: felt(tx.class_hash).into(),
    }
}

fn l1_handler_transaction(tx: &p::L1HandlerTransaction) -> mp_transactions::HandleL1MessageTransaction {
    mp_transactions::HandleL1MessageTransaction {
        nonce: u64::try_from(felt(tx.nonce.unwrap())).unwrap(),
        contract_address: felt(tx.contract_address).into(),
        entry_point_selector: felt(tx.entry_point_selector).into(),
        calldata: tx.calldata.iter().copied().map(felt).map(Into::into).collect(),
    }
}

fn fee(felt: starknet_ff::FieldElement) -> u128 {
    // FIXME: WHY IS THIS CONVERTION EVEN A THING??
    let _ = felt;
    0
}

fn events(receipts: &[p::ConfirmedTransactionReceipt]) -> Vec<starknet_api::transaction::Event> {
    receipts.iter().flat_map(|r| &r.events).map(event).collect()
}

fn event(event: &p::Event) -> starknet_api::transaction::Event {
    use starknet_api::transaction::{Event, EventContent, EventData, EventKey};

    Event {
        from_address: contract_address(event.from_address),
        content: EventContent {
            keys: event.keys.iter().copied().map(felt).map(EventKey).collect(),
            data: EventData(event.data.iter().copied().map(felt).collect()),
        },
    }
}

fn commitments(
    transactions: &[mp_transactions::Transaction],
    events: &[starknet_api::transaction::Event],
    block_number: u64,
) -> (StarkFelt, StarkFelt) {
    use mp_hashers::pedersen::PedersenHasher;

    let chain_id = chain_id();

    let (a, b) = mp_commitments::calculate_commitments::<PedersenHasher>(transactions, events, chain_id, block_number);

    (a.into(), b.into())
}

fn chain_id() -> mp_felt::Felt252Wrapper {
    starknet_ff::FieldElement::from_byte_slice_be(b"SN_MAIN").unwrap().into()
}

fn felt(field_element: starknet_ff::FieldElement) -> starknet_api::hash::StarkFelt {
    starknet_api::hash::StarkFelt::new(field_element.to_bytes_be()).unwrap()
}

fn contract_address(field_element: starknet_ff::FieldElement) -> starknet_api::api_core::ContractAddress {
    starknet_api::api_core::ContractAddress(starknet_api::api_core::PatriciaKey(felt(field_element)))
}
