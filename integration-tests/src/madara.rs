#![cfg(test)]

use anyhow::anyhow;
use sp_core::crypto::Ss58Codec;
use sp_core::Bytes;
use sp_keyring::sr25519::Keyring;
use subxt::dynamic::Value;
use subxt::tx::SubmittableExtrinsic;
use subxt::utils::AccountId32;
use subxt::{rpc_params, OnlineClient, SubstrateConfig};

use crate::codegen::aura::api;
use crate::codegen::aura::api::balances::events::Transfer;
use crate::codegen::aura::api::runtime_types::aura_runtime::RuntimeCall;
use crate::codegen::aura::api::runtime_types::frame_system::pallet::Call;
use crate::codegen::aura::api::runtime_types::sp_weights::weight_v2::Weight;
use crate::codegen::aura::api::system::events::CodeUpdated;

#[tokio::test]
async fn test_all_features() -> Result<(), anyhow::Error> {
    simple_transfer().await?;
    // runtime_upgrades().await?;
    revert_blocks().await?;
    Ok(())
}

async fn simple_transfer() -> Result<(), anyhow::Error> {
    let client = OnlineClient::<SubstrateConfig>::from_url("ws://127.0.0.1:9944").await?;

    let bob = AccountId32::from(Keyring::Bob.public());
    let alice = AccountId32::from(Keyring::Alice.public());

    let addr = api::storage().system().account(alice.clone());
    let old = client.storage().at_latest().await?.fetch(&addr).await?.expect("Account should exist").data.free;

    let call = client.tx().call_data(&api::tx().balances().transfer(bob.clone().into(), old / 2))?;

    let extrinsic: Bytes = client
        .rpc()
        .request(
            "simnode_authorExtrinsic",
            // author an extrinsic from alice
            rpc_params![Bytes::from(call), Keyring::Alice.to_account_id().to_ss58check()],
        )
        .await?;

    let submittable = SubmittableExtrinsic::from_bytes(client.clone(), extrinsic.0);
    let events = submittable.submit_and_watch().await?.wait_for_finalized_success().await?;
    let transfer = events
        .find::<Transfer>()
        .collect::<Result<Vec<_>, subxt::Error>>()?
        .pop()
        .ok_or_else(|| anyhow!("transfer event not found!"))?;

    // assert that the event was emitted
    assert_eq!(transfer, Transfer { from: alice.clone(), to: bob.clone(), amount: old / 2 });

    // confirm that state has changed
    let addr = api::storage().system().account(&alice);
    let new = client.storage().at_latest().await?.fetch(&addr).await?.expect("Account should exist").data.free;

    assert!(new <= old / 2);

    Ok(())
}

// async fn runtime_upgrades() -> Result<(), anyhow::Error> {
// 	let client = OnlineClient::<SubstrateConfig>::from_url("ws://127.0.0.1:9944").await?;

// 	let old_version = client.rpc().runtime_version(None).await?;
// 	assert_eq!(old_version.spec_version, 1);

// 	let code = include_bytes!("../../../assets/aura-runtime-upgrade.wasm").to_vec();

// 	let call = client.tx().call_data(&api::tx().sudo().sudo_unchecked_weight(
// 		RuntimeCall::System(Call::set_code { code }),
// 		Weight { ref_time: 0, proof_size: 0 },
// 	))?;

// 	let extrinsic: Bytes = client
// 		.rpc()
// 		.request(
// 			"simnode_authorExtrinsic",
// 			// author an extrinsic from the sudo account.
// 			rpc_params![Bytes::from(call), Keyring::Alice.to_account_id().to_ss58check()],
// 		)
// 		.await?;
// 	let submittable = SubmittableExtrinsic::from_bytes(client.clone(), extrinsic.0);
// 	let events = submittable.submit_and_watch().await?.wait_for_finalized_success().await?;
// 	// assert that the event was emitted
// 	events
// 		.find::<CodeUpdated>()
// 		.collect::<Result<Vec<_>, subxt::Error>>()?
// 		.pop()
// 		.ok_or_else(|| anyhow!("transfer event not found!"))?;

// 	// assert the version
// 	let new_version = client.rpc().runtime_version(None).await?;
// 	assert_eq!(new_version.spec_version, 1000);

// 	// try to create 10 blocks to assert that the runtime still works.
// 	for _ in 0..10 {
// 		client
// 			.rpc()
// 			.request::<Value>("engine_createBlock", rpc_params![true, true])
// 			.await?;
// 	}

// 	Ok(())
// }

async fn revert_blocks() -> Result<(), anyhow::Error> {
    let client = OnlineClient::<SubstrateConfig>::from_url("ws://127.0.0.1:9944").await?;
    let old_header = client.rpc().header(None).await?.ok_or_else(|| anyhow!("Header not found!"))?;
    let n = 10;

    for _ in 0..n {
        client.rpc().request::<Value>("engine_createBlock", rpc_params![true, true]).await?;
    }
    let new_header = client.rpc().header(None).await?.ok_or_else(|| anyhow!("Header not found!"))?;

    assert_eq!(old_header.number + n, new_header.number);

    let revert = n / 2;

    client.rpc().request::<()>("simnode_revertBlocks", rpc_params![revert]).await?;

    let new_header = client.rpc().header(None).await?.ok_or_else(|| anyhow!("Header not found!"))?;

    assert_eq!(old_header.number + revert, new_header.number);

    Ok(())
}
