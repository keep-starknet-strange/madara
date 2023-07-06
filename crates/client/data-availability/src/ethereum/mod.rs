use std::sync::Arc;

use ethers::core::utils::Anvil;
use ethers::prelude::{abigen, SignerMiddleware};
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, I256, U256};

pub const _STARKNET_MAINNET_CC_ADDRESS: &str = "0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4";
pub const STARKNET_GOERLI_CC_ADDRESS: &str = "0xde29d060D45901Fb19ED6C6e959EB22d8626708e";

// TODO:
// - remove unwraps
// - test sequencer address
// - make chain configurable
pub async fn publish_data(eth_node: &str, _sequencer_address: &[u8], state_diff: Vec<U256>) {
    abigen!(
        STARKNET,
        r#"[
            function updateState(uint256[] calldata programOutput, uint256 onchainDataHash, uint256 onchainDataSize) external
        ]"#,
    );

    let provider = Provider::<Http>::try_from(eth_node).unwrap();

    let anvil = Anvil::new().spawn();
    let from_wallet: LocalWallet = anvil.keys()[0].clone().into();

    let address: Address = STARKNET_GOERLI_CC_ADDRESS.parse().unwrap();
    let signer = Arc::new(SignerMiddleware::new(provider, from_wallet.with_chain_id(anvil.chain_id())));
    let contract = STARKNET::new(address, signer);

    let tx = contract.update_state(state_diff, U256::default(), U256::default());
    let pending_tx = tx.send().await.unwrap();
    let minted_tx = pending_tx.await.unwrap();
    log::info!("State Update: {:?}", minted_tx);
}

pub async fn last_proven_block(eth_node: &str) -> Result<I256, String> {
    abigen!(
        STARKNET,
        r#"[
            function stateBlockNumber() external view returns (int256)
        ]"#,
    );

    let provider = Provider::<Http>::try_from(eth_node).unwrap();
    let client = Arc::new(provider);

    let address: Address = STARKNET_GOERLI_CC_ADDRESS.parse().unwrap();
    let contract = STARKNET::new(address, client);
    contract.state_block_number().call().await.map_err(|e| format!("ethereum contract err: {e}"))
}
