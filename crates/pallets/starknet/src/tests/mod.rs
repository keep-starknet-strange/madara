use mp_starknet::execution::types::{
    CallEntryPointWrapper, ClassHashWrapper, ContractAddressWrapper, EntryPointTypeWrapper, Felt252Wrapper,
};
use mp_starknet::transaction::types::{Transaction, TxType};
use sp_core::bounded_vec;

mod account_helper;
mod call_contract;
mod declare_tx;
mod deploy_account_tx;
mod erc20;
mod invoke_tx;
mod l1_message;
mod query_tx;
mod sequencer_address;

mod constants;
mod mock;
mod utils;

pub fn get_invoke_dummy() -> Transaction {
    let hash =
        Felt252Wrapper::from_hex_be("0x06fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212").unwrap();
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        Felt252Wrapper::from_hex_be("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    );
    let sender_address =
        Felt252Wrapper::from_hex_be("02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77").unwrap();
    let nonce = Felt252Wrapper::ZERO;

    let call_entrypoint = CallEntryPointWrapper {
        class_hash: Some(
            ClassHashWrapper::from_hex_be("0x03bcec8de953ba8e305e2ce2db52c91504aefa7c56c91211873b4d6ba36e8c32")
                .unwrap(),
        ),
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: None,
        calldata: bounded_vec!(
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
            Felt252Wrapper::from_hex_be("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap()
        ),
        storage_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        caller_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        initial_gas: Felt252Wrapper::from_dec_str("0123").unwrap(),
    };

    Transaction {
        tx_type: TxType::Invoke,
        version: 1,
        hash,
        signature,
        sender_address,
        nonce,
        call_entrypoint,
        contract_class: None,
        contract_address_salt: None,
        ..Transaction::default()
    }
}

fn get_invoke_argent_dummy() -> Transaction {
    let hash =
        Felt252Wrapper::from_hex_be("0x0179d464009de591507b8b6213286b83a4d3b69bf5b4ad82aeeee245a9b363f4").unwrap();
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        Felt252Wrapper::from_hex_be("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    );
    let sender_address =
        Felt252Wrapper::from_hex_be("02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5").unwrap();
    let nonce = Felt252Wrapper::ZERO;

    let call_entrypoint = CallEntryPointWrapper {
        class_hash: Some(
            ClassHashWrapper::from_hex_be("0x03bcec8de953ba8e305e2ce2db52c91504aefa7c56c91211873b4d6ba36e8c32")
                .unwrap(),
        ),
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: None,
        calldata: bounded_vec!(
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
            Felt252Wrapper::from_hex_be("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(),
        ),
        storage_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        caller_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        initial_gas: Felt252Wrapper::from_dec_str("0123").unwrap(),
    };

    Transaction {
        tx_type: TxType::Invoke,
        version: 1,
        hash,
        signature,
        sender_address,
        nonce,
        call_entrypoint,
        contract_class: None,
        contract_address_salt: None,
        ..Transaction::default()
    }
}

fn get_invoke_braavos_dummy() -> Transaction {
    let hash =
        Felt252Wrapper::from_hex_be("0x03345fb7d038a238a8d9e8d0cec1c787cbc15ebd3a44f6fb82991c36dca94286").unwrap();
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        Felt252Wrapper::from_hex_be("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    );
    let sender_address =
        Felt252Wrapper::from_hex_be("05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122").unwrap();
    let nonce = Felt252Wrapper::ZERO;

    let call_entrypoint = CallEntryPointWrapper {
        class_hash: Some(
            ClassHashWrapper::from_hex_be("0x03bcec8de953ba8e305e2ce2db52c91504aefa7c56c91211873b4d6ba36e8c32")
                .unwrap(),
        ),
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: None,
        calldata: bounded_vec!(
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
            Felt252Wrapper::from_hex_be("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(),
        ),
        storage_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        caller_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        initial_gas: Felt252Wrapper::from_dec_str("0123").unwrap(),
    };

    Transaction {
        tx_type: TxType::Invoke,
        version: 1,
        hash,
        signature,
        sender_address,
        nonce,
        call_entrypoint,
        contract_class: None,
        contract_address_salt: None,
        ..Transaction::default()
    }
}

fn get_invoke_emit_event() -> Transaction {
    let hash =
        Felt252Wrapper::from_hex_be("0x06fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212").unwrap();
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        Felt252Wrapper::from_hex_be("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    );
    let sender_address =
        Felt252Wrapper::from_hex_be("01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0").unwrap();
    let nonce = Felt252Wrapper::ZERO;

    let call_entrypoint = CallEntryPointWrapper {
        class_hash: Some(
            ClassHashWrapper::from_hex_be("0x03bcec8de953ba8e305e2ce2db52c91504aefa7c56c91211873b4d6ba36e8c32")
                .unwrap(),
        ),
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: None,
        calldata: bounded_vec!(
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
            Felt252Wrapper::from_hex_be("0x00966af5d72d3975f70858b044c77785d3710638bbcebbd33cc7001a91025588").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap()
        ),
        storage_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        caller_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        initial_gas: Felt252Wrapper::from_dec_str("0123").unwrap(),
    };

    Transaction {
        tx_type: TxType::Invoke,
        version: 1,
        hash,
        signature,
        sender_address,
        nonce,
        call_entrypoint,
        contract_class: None,
        contract_address_salt: None,
        ..Transaction::default()
    }
}

fn get_invoke_nonce() -> Transaction {
    let hash =
        Felt252Wrapper::from_hex_be("0x06fc3466f58b5c6aaa6633d48702e1f2048fb96b7de25f2bde0bce64dca1d212").unwrap();
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        Felt252Wrapper::from_hex_be("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    );
    let sender_address =
        Felt252Wrapper::from_hex_be("02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77").unwrap();
    let nonce = Felt252Wrapper::ONE;

    let call_entrypoint = CallEntryPointWrapper {
        class_hash: Some(
            ClassHashWrapper::from_hex_be("0x03bcec8de953ba8e305e2ce2db52c91504aefa7c56c91211873b4d6ba36e8c32")
                .unwrap(),
        ),
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: None,
        calldata: bounded_vec!(
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
            Felt252Wrapper::from_hex_be("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
            Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(),
        ),
        storage_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        caller_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        initial_gas: Felt252Wrapper::from_dec_str("0123").unwrap(),
    };

    Transaction {
        tx_type: TxType::Invoke,
        version: 1,
        hash,
        signature,
        sender_address,
        nonce,
        call_entrypoint,
        contract_class: None,
        contract_address_salt: None,
        ..Transaction::default()
    }
}

fn get_storage_read_write_dummy() -> Transaction {
    let hash =
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    let signature = bounded_vec!();
    let sender_address =
        Felt252Wrapper::from_hex_be("02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77").unwrap();
    let nonce = Felt252Wrapper::ZERO;

    let call_entrypoint = CallEntryPointWrapper {
        class_hash: Some(
            ClassHashWrapper::from_hex_be("025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918").unwrap(),
        ),
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: None,
        calldata: bounded_vec!(
            Felt252Wrapper::from_hex_be("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
            Felt252Wrapper::from_hex_be("03b097c62d3e4b85742aadd0dfb823f96134b886ec13bda57b68faf86f294d97").unwrap(),
            Felt252Wrapper::from_hex_be("0000000000000000000000000000000000000000000000000000000000000002").unwrap(),
            Felt252Wrapper::from_hex_be("0000000000000000000000000000000000000000000000000000000000000019").unwrap(),
            Felt252Wrapper::from_hex_be("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        ),
        storage_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        caller_address: ContractAddressWrapper::from_hex_be(
            "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        )
        .unwrap(),
        initial_gas: Felt252Wrapper::from_dec_str("0123").unwrap(),
    };

    Transaction {
        tx_type: TxType::Invoke,
        version: 1,
        hash,
        signature,
        sender_address,
        nonce,
        call_entrypoint,
        contract_class: None,
        contract_address_salt: None,
        ..Transaction::default()
    }
}
