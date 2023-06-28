use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
    types::Address,
};

use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use futures::StreamExt;
use sc_client_api::client::BlockchainEvents;
use serde::Deserialize;
use sp_api::ProvideRuntimeApi;
use sp_core::H256;
use sp_runtime::traits::Block as BlockT;
use uuid::Uuid;

pub const TEST_CAIRO_PIE_BASE64: &str = "UEsDBBQAAAAIAAAAIQBGGR6jGAEAAEsCAAANAAAAbWV0YWRhdGEuanNvbnWR3WqDQBCFX0W8zsX87M7O9lVCCDbZBqEa0RVCgu/e0aaYgL1bznwz5xx9lF1/vfRVU34Uj/Jc5coeeyfIHr0gEaojz87tCgTYFR5V1YMTYIwRgjoTN3F6ww1xGjyy/IPzG64Roo0c0yxGjcpeOARx3q6YyGbAziNoEGEMjM7WDrui/Bzr71y3w9yjvI65G3M5601Vt6aZi1Wum2RvFjQ3CqoilgnJ4sXAZApD9BAAwQsxQiD0zOijSIyGQCSbWA0NBAROcbK7fcrHr+44pEuT2rx80bo9p9tsZeOhvs+uMK0p/9hhgZ9pX/do3eNpXky3dBpzfW03bXDFw7Q0XX7uJgsri/R7OvfVa6L94VmqO21ecK+lph9QSwMEFAAAAAgAAAAhAEMqWVOVAAAAcAMAAAoAAABtZW1vcnkuYmlujZIxDsIwDEVtp6QULsHExCE69iDcI957sV6qlAq+UCphOz9Lhqf/HMVEnyjpVljzSEYY3NMCEPn1rYVUJotLjd4O3BJ4T5WXHW9u9PbgHux7z5VXHO9wmO9uchdwydfSFdyrbPuZby68W78X8zHKNac2J4c+MbkELtoXzBVyHfqiPWD0Ne2Lxv8r6Iu4jL5UTfEn+gZQSwMEFAAAAAgAAAAhAKwgWIQtAAAAMwAAABQAAABhZGRpdGlvbmFsX2RhdGEuanNvbqtWyi8tKSgtiU8qzcwpycxTslKoVipITE8tBrFqdRSUEktKijKTSkugIrW1AFBLAwQUAAAACAAAACEAIxC1mksAAABWAAAAGAAAAGV4ZWN1dGlvbl9yZXNvdXJjZXMuanNvbqtWSirNzCnJzIvPzCsuScxLTo1Pzi/NK0ktUrJSqFbKLy0pKC2Jh6oBChnX6igo5cUXl6QWFAO5FmBebmpuflFlfEZ+TipI0KAWAFBLAwQUAAAACAAAACEA2YDFkhYAAAAUAAAADAAAAHZlcnNpb24uanNvbqtWSk7MLMqPL8hMVbJSUDLUM1SqBQBQSwECFAMUAAAACAAAACEARhkeoxgBAABLAgAADQAAAAAAAAAAAAAAgAEAAAAAbWV0YWRhdGEuanNvblBLAQIUAxQAAAAIAAAAIQBDKllTlQAAAHADAAAKAAAAAAAAAAAAAACAAUMBAABtZW1vcnkuYmluUEsBAhQDFAAAAAgAAAAhAKwgWIQtAAAAMwAAABQAAAAAAAAAAAAAAIABAAIAAGFkZGl0aW9uYWxfZGF0YS5qc29uUEsBAhQDFAAAAAgAAAAhACMQtZpLAAAAVgAAABgAAAAAAAAAAAAAAIABXwIAAGV4ZWN1dGlvbl9yZXNvdXJjZXMuanNvblBLAQIUAxQAAAAIAAAAIQDZgMWSFgAAABQAAAAMAAAAAAAAAAAAAACAAeACAAB2ZXJzaW9uLmpzb25QSwUGAAAAAAUABQA1AQAAIAMAAAAA";
pub const TEST_CAIRO_FACT: &str = "0x99f8c8b3efce1cb3b53ce44fd5e8339a1299be480cc6e4599d107f69666eb7bb";
pub const TEST_JOB_ID: &str = "61a99af0-dcbb-47c9-8350-a08e488af073";
pub const LAMBDA_URL: &str = "https://testnet.provingservice.io";
// pub const LAMBDA_MAX_PIE_MB: u64 = 20 * 2**20;

pub struct DataAvailabilityWorker<B, C>(PhantomData<(B, C)>);

impl<B, C> DataAvailabilityWorker<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C: BlockchainEvents<B> + 'static,
{
    pub async fn prove_current_block(client: Arc<C>, madara_backend: Arc<mc_db::Backend<B>>) {
        let mut notification_st = client.import_notification_stream();

        while let Some(notification) = notification_st.next().await {
            // Run the StarkNet OS for Block

            // Store the DA facts
            let _res = madara_backend
                .fact()
                .store_block_facts(&notification.hash, vec![H256::from_str(TEST_CAIRO_FACT).unwrap()]);
			
			// Submit the StarkNet OS PIE
			if let Ok(job_resp) = submit_pie(TEST_CAIRO_PIE_BASE64) {
				log::info!("Submitted job id: {}", job_resp.cairo_job_key);

				// Store the cairo job key
				let _res = madara_backend.fact().update_cairo_job(&notification.hash, Uuid::from_str(TEST_JOB_ID).unwrap());
			}
        }
    }
}

impl<B, C> DataAvailabilityWorker<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C: BlockchainEvents<B> + 'static,
{
    pub async fn update_state(client: Arc<C>, madara_backend: Arc<mc_db::Backend<B>>) {
        let mut notification_st = client.import_notification_stream();

        while let Some(notification) = notification_st.next().await {
            // Query last proven block
			// let res = madara_backend.fact().last_proved_block().unwrap();
			starknet_last_proven_block().await;
            // log::info!("Last proved block: {}", res);

			// Check the associated job status
			if let Ok(job_resp) = get_status(TEST_JOB_ID) {
				log::info!("Job status: {}", job_resp.status);
				// TODO: use fact db enum type
				if job_resp.status == "ONCHAIN" {
					// Fetch DA Facts for block
					let _res = madara_backend.fact().block_facts(&notification.hash).unwrap();

				}
			}
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct CairoJobResponse {
    pub cairo_job_key: String,
    pub version: u64,
}

// Send zipped CairoPie to SHARP
//  - /add_job {"cairo_pie": base64.b64encode(cairo_pie.serialize()).decode("ascii")}
pub fn submit_pie(pie: &str) -> Result<CairoJobResponse, String> {
    let data = serde_json::json!({ "cairo_pie": pie });
    let data = serde_json::json!({ "action": "add_job", "request": data });
    let payload: serde_json::Value = serde_json::from_value(data).unwrap();

    let resp = reqwest::blocking::Client::new().post(LAMBDA_URL).json(&payload).send().unwrap();

    match resp.status() {
        reqwest::StatusCode::OK => Ok(resp.json::<CairoJobResponse>().unwrap()),
        _ => Err(String::from("could not submit pie")),
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CairoStatusResponse {
    pub status: String,
    #[serde(rename = "validation_done")]
    pub validation_done: bool,
    pub version: u64,
}

// Check on job
// - /get_status {"cairo_job_key": job_key}
pub fn get_status(job_key: &str) -> Result<CairoStatusResponse, String> {
    let data = serde_json::json!({ "cairo_job_key": job_key });
    let data = serde_json::json!({ "action": "get_status", "request": data });
    let payload: serde_json::Value = serde_json::from_value(data).unwrap();

    let resp = reqwest::blocking::Client::new().post(LAMBDA_URL).json(&payload).send().unwrap();

    match resp.status() {
        reqwest::StatusCode::OK => Ok(resp.json::<CairoStatusResponse>().unwrap()),
        _ => Err(String::from("could not get job status")),
    }
}

// async fn publish_data(sender_id: &[u8], data: &[u8]) -> Result<(), &str> {
    // abigen!(
    //     STARKNET,
    //     r#"[
    //         function updateState(uint256[] calldata programOutput, uint256 onchainDataHash, uint256 onchainDataSize) external
    //     ]"#,
    // );

    // const RPC_URL: &str = "https://eth-mainnet.g.alchemy.com/v2/<TODO: config>";
    // pub const STARKNET_MAINNET_CC_ADDRESS: &str = "0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4";
    // // pub const STARKNET_GOERLI_CC_ADDRESS: &str = "0xde29d060D45901Fb19ED6C6e959EB22d8626708e";
    
    // let provider = Provider::<Http>::try_from(RPC_URL).unwrap();
    // let client = Arc::new(provider);

    // let address: Address = STARKNET_MAINNET_CC_ADDRESS.parse().unwrap();
    // let contract = STARKNET::new(address, client);
    // if let Ok(state_block_number) = contract.state_block_number().call().await {
    //     log::info!("State Block Number {state_block_number:?}");
    // }
// }

pub async fn starknet_last_proven_block() {
    abigen!(
        STARKNET,
        r#"[
            function stateBlockNumber() external view returns (int256)
        ]"#,
    );

    const RPC_URL: &str = "https://eth-mainnet.g.alchemy.com/v2/<TODO: config>";
    pub const STARKNET_MAINNET_CC_ADDRESS: &str = "0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4";
    // pub const STARKNET_GOERLI_CC_ADDRESS: &str = "0xde29d060D45901Fb19ED6C6e959EB22d8626708e";
    
    let provider = Provider::<Http>::try_from(RPC_URL).unwrap();
    let client = Arc::new(provider);

    let address: Address = STARKNET_MAINNET_CC_ADDRESS.parse().unwrap();
    let contract = STARKNET::new(address, client);
    if let Ok(state_block_number) = contract.state_block_number().call().await {
        log::info!("State Block Number {state_block_number:?}");
    }
}
