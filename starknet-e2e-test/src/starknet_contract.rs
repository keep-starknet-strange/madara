use ethers::prelude::abigen;
use ethers::types::{Address, Bytes, U256};
use ethers::utils::keccak256;
use mc_settlement::ethereum::convert_felt_to_u256;
use mp_felt::Felt252Wrapper;
use mp_messages::{MessageL1ToL2, MessageL2ToL1};
use mp_snos_output::SnosCodec;
use starknet_api::hash::StarkFelt;
use starknet_ff::FieldElement;

use crate::ethereum_sandbox::EthereumSandbox;

const STARKNET_VALIDIUM: &str = include_str!("../contracts/artifacts/Starknet.json");
const UNSAFE_PROXY: &str = include_str!("../contracts/artifacts/UnsafeProxy.json");

// Starknet core contract cannot be initialized directly hence we use a proxy for that.
// In order to initialize the contract we need to provide the following data:
//      0. Sub-proxy contracts - none in our case
//      1. External initializer contract (EIC) - zero in our case
//      2. Program hash - non-zero, otherwise will be considered invalid
//      3. Verifier address
//      4. Config hash
//      5. Global state root (genesis)
//      6. Block number (genesis)
//      7. Block hash (genesis)
//
// Once we have an initialized contract we also need to assign operator -
// the account we will use for updating the state.
abigen!(
    StarknetInitializer,
    r#"[
        function initialize(bytes calldata data) external
        function registerOperator(address newOperator) external
    ]"#,
);

// Starknet messaging interface for testing purposes.
abigen!(
    StarknetMessaging,
    r#"[
        function sendMessageToL2(uint256 toAddress, uint256 selector, uint256[] calldata payload) external payable returns (bytes32, uint256)
        function l1ToL2Messages(bytes32 msgHash) external view returns (uint256)
        function l2ToL1Messages(bytes32 msgHash) external view returns (uint256)
    ]"#,
);

#[derive(Clone, Debug, Default)]
pub struct InitData {
    pub program_hash: StarkFelt,
    pub verifier_address: StarkFelt,
    pub config_hash: StarkFelt,
    pub state_root: StarkFelt,
    pub block_number: StarkFelt,
    pub block_hash: StarkFelt,
}

impl From<InitData> for Bytes {
    // No dynamic fields, so the encoding is pretty straightforward:
    //
    //      abi.encode(data, (uint256, address, uint256, StarknetState.State));
    //      where struct State {
    //          uint256 globalRoot;
    //          int256 blockNumber;
    //          uint256 blockHash;
    //      }
    fn from(val: InitData) -> Self {
        let mut bytes = [0u8; 7 * 32];
        // Recall:
        //  * None sub-proxy contracts
        //  * First 32 bytes are for the EIC - not specified
        bytes[32..64].copy_from_slice(val.program_hash.bytes());
        bytes[64..96].copy_from_slice(val.verifier_address.bytes());
        bytes[96..128].copy_from_slice(val.config_hash.bytes());
        bytes[128..160].copy_from_slice(val.state_root.bytes());
        bytes[160..192].copy_from_slice(val.block_number.bytes());
        bytes[192..224].copy_from_slice(val.block_hash.bytes());
        bytes.into()
    }
}

impl InitData {
    /// Use the same config as in Starknet Goerli testnet
    pub fn sn_goerli() -> Self {
        Self {
            // See SN_OS_PROGRAM_HASH constant
            program_hash: StarkFelt::from(Felt252Wrapper::from(
                FieldElement::from_hex_be("0x41fc2a467ef8649580631912517edcab7674173f1dbfa2e9b64fbcd82bc4d79").unwrap(),
            )),
            // Hash version:        SN_OS_CONFIG_HASH_VERSION (settlement)
            // Chain ID:            SN_GOERLI_CHAIN_ID (pallet config)
            // Fee token address:   0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7 (genesis config)
            config_hash: StarkFelt::from(Felt252Wrapper::from(
                FieldElement::from_hex_be("0x036f5e4ea4dd042801c8841e3db8e654124305da0f11824fc1db60c405dbb39f")
                    .unwrap(),
            )),
            ..Default::default()
        }
    }

    pub fn one() -> Self {
        Self { program_hash: 1u64.into(), config_hash: 1u64.into(), ..Default::default() }
    }
}

pub struct StarknetContract {
    address: Address,
}

impl StarknetContract {
    pub async fn deploy(sandbox: &EthereumSandbox) -> Self {
        // First we deploy the Starknet contract (no explicit constructor)
        let snos_contract = sandbox.deploy(STARKNET_VALIDIUM, ()).await;

        // Then we deploy a simple delegate proxy to interact with Starknet contract (initialized with its
        // address)
        let proxy_contract = sandbox.deploy(UNSAFE_PROXY, snos_contract.address()).await;

        // We will use proxy to interact with the Starknet core contract
        Self { address: proxy_contract.address() }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub async fn initialize(&self, sandbox: &EthereumSandbox, data: InitData) {
        // This is the initialization interface
        let initializer = StarknetInitializer::new(self.address, sandbox.client());

        // 1. Provide Starknet OS program/config and genesis state
        initializer
            .initialize(data.into())
            .send()
            .await
            .expect("Failed to call `initialize`")
            .await
            .expect("Ethereum poll update error")
            .unwrap();

        // 2. Add our EOA as Starknet operator
        initializer
            .register_operator(sandbox.address())
            .send()
            .await
            .expect("Failed to call `register_operator`")
            .await
            .expect("Ethereum poll update error")
            .unwrap();
    }

    pub async fn send_message_to_l2(&self, sandbox: &EthereumSandbox, message: &MessageL1ToL2) {
        let messaging = StarknetMessaging::new(self.address, sandbox.client());

        messaging.send_message_to_l2(
            convert_felt_to_u256(message.to_address.0.0),
            convert_felt_to_u256(message.selector),
            message.payload.clone().into_iter().map(convert_felt_to_u256).collect())
            .value(1) // L1 message fee must be between 0 and 1 ether
            .send()
            .await
            .expect("Failed to call `send_message_to_l2`")
            .await
            .expect("Ethereum poll update error")
            .unwrap();
    }

    pub async fn message_to_l1_exists(&self, sandbox: &EthereumSandbox, message: &MessageL2ToL1) -> bool {
        let messaging = StarknetMessaging::new(self.address, sandbox.client());

        let mut payload: Vec<u8> = Vec::new();
        message.clone().into_encoded_vec().into_iter().for_each(|felt| payload.append(&mut felt.bytes().to_vec()));

        let msg_hash = keccak256(payload);
        let res = messaging.l_2_to_l1_messages(msg_hash).call().await.expect("Failed to call `l_2_to_l1_messages`");

        res != U256::zero()
    }

    pub async fn message_to_l2_exists(&self, sandbox: &EthereumSandbox, message: &MessageL1ToL2) -> bool {
        let messaging = StarknetMessaging::new(self.address, sandbox.client());

        let mut payload: Vec<u8> = Vec::new();
        message.clone().into_encoded_vec().into_iter().for_each(|felt| payload.append(&mut felt.bytes().to_vec()));

        let msg_hash = keccak256(payload);
        let res = messaging.l_1_to_l2_messages(msg_hash).call().await.expect("Failed to call `l_2_to_l1_messages`");

        res != U256::zero()
    }
}
