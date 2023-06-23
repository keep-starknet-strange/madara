use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::{InvokeTransaction, Transaction};
use sp_core::bounded_vec;

use self::mock::Starknet;

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
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        Felt252Wrapper::from_hex_be("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    );
    let sender_address = Felt252Wrapper::from_hex_be(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap();
    let nonce = Felt252Wrapper::ZERO;
    let calldata = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
        Felt252Wrapper::from_hex_be("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap()
    );

    InvokeTransaction {
        version: 1,
        sender_address,
        calldata,
        nonce,
        signature,
        max_fee: Felt252Wrapper::from(u128::MAX),
    }
    .from_invoke(Starknet::chain_id())
}

fn get_invoke_argent_dummy() -> Transaction {
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        Felt252Wrapper::from_hex_be("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    );
    let sender_address =
        Felt252Wrapper::from_hex_be("02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5").unwrap();
    let nonce = Felt252Wrapper::ZERO;
    let calldata = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
        Felt252Wrapper::from_hex_be("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(),
    );

    InvokeTransaction {
        version: 1,
        sender_address,
        calldata,
        nonce,
        signature,
        max_fee: Felt252Wrapper::from(u128::MAX),
    }
    .from_invoke(Starknet::chain_id())
}

fn get_invoke_braavos_dummy() -> Transaction {
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        Felt252Wrapper::from_hex_be("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    );
    let sender_address =
        Felt252Wrapper::from_hex_be("05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122").unwrap();
    let nonce = Felt252Wrapper::ZERO;
    let calldata = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
        Felt252Wrapper::from_hex_be("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(),
    );

    InvokeTransaction {
        version: 1,
        sender_address,
        calldata,
        nonce,
        signature,
        max_fee: Felt252Wrapper::from(u128::MAX),
    }
    .from_invoke(Starknet::chain_id())
}

fn get_invoke_emit_event() -> Transaction {
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        Felt252Wrapper::from_hex_be("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    );
    let sender_address =
        Felt252Wrapper::from_hex_be("01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0").unwrap();
    let nonce = Felt252Wrapper::ZERO;
    let calldata = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
        Felt252Wrapper::from_hex_be("0x00966af5d72d3975f70858b044c77785d3710638bbcebbd33cc7001a91025588").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap()
    );

    InvokeTransaction {
        version: 1,
        sender_address,
        calldata,
        nonce,
        signature,
        max_fee: Felt252Wrapper::from(u128::MAX),
    }
    .from_invoke(Starknet::chain_id())
}

fn get_invoke_nonce() -> Transaction {
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x00f513fe663ffefb9ad30058bb2d2f7477022b149a0c02fb63072468d3406168").unwrap(),
        Felt252Wrapper::from_hex_be("0x02e29e92544d31c03e89ecb2005941c88c28b4803a3647a7834afda12c77f096").unwrap(),
    );
    let sender_address = Felt252Wrapper::from_hex_be(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap();
    let nonce = Felt252Wrapper::ONE;
    let calldata = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
        Felt252Wrapper::from_hex_be("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(),
    );

    InvokeTransaction {
        version: 1,
        sender_address,
        calldata,
        nonce,
        signature,
        max_fee: Felt252Wrapper::from(u128::MAX),
    }
    .from_invoke(Starknet::chain_id())
}

fn get_storage_read_write_dummy() -> Transaction {
    let signature = bounded_vec!();
    let sender_address = Felt252Wrapper::from_hex_be(constants::BLOCKIFIER_ACCOUNT_ADDRESS).unwrap();
    let nonce = Felt252Wrapper::ZERO;
    let calldata = bounded_vec!(
        Felt252Wrapper::from_hex_be("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
        Felt252Wrapper::from_hex_be("03b097c62d3e4b85742aadd0dfb823f96134b886ec13bda57b68faf86f294d97").unwrap(),
        Felt252Wrapper::from_hex_be("0000000000000000000000000000000000000000000000000000000000000002").unwrap(),
        Felt252Wrapper::from_hex_be("0000000000000000000000000000000000000000000000000000000000000019").unwrap(),
        Felt252Wrapper::from_hex_be("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
    );

    InvokeTransaction {
        version: 1,
        sender_address,
        calldata,
        nonce,
        signature,
        max_fee: Felt252Wrapper::from(u128::MAX),
    }
    .from_invoke(Starknet::chain_id())
}

fn get_invoke_openzeppelin_dummy() -> Transaction {
    let signature = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x01ef15c18599971b7beced415a40f0c7deacfd9b0d1819e03d723d8bc943cfca").unwrap(),
        Felt252Wrapper::from_hex_be("0x004f0481f89eae56dec538294bde0bf84bba526517dd9ff7dcb2a22628ee4d9e").unwrap(),
    );
    let sender_address =
        Felt252Wrapper::from_hex_be("06e2616a2dceff4355997369246c25a78e95093df7a49e5ca6a06ce1544ffd50").unwrap();
    let nonce = Felt252Wrapper::ZERO;
    let calldata = bounded_vec!(
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), /* call_array_len */
        Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(), /* to */
        Felt252Wrapper::from_hex_be("0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc").unwrap(), /* selector */
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(), /* data offset */
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), /* data length */
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), /* calldata_len */
        Felt252Wrapper::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000019").unwrap(), /* calldata */
    );

    InvokeTransaction {
        version: 1,
        sender_address,
        calldata,
        nonce,
        signature,
        max_fee: Felt252Wrapper::from(u128::MAX),
    }
    .from_invoke(Starknet::chain_id())
}
