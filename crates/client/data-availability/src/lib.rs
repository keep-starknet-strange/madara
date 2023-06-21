pub const CAIRO_PIE_BASE64: &str = "UEsDBBQAAAAIAAAAIQBGGR6jGAEAAEsCAAANAAAAbWV0YWRhdGEuanNvbnWR3WqDQBCFX0W8zsX87M7O9lVCCDbZBqEa0RVCgu/e0aaYgL1bznwz5xx9lF1/vfRVU34Uj/Jc5coeeyfIHr0gEaojz87tCgTYFR5V1YMTYIwRgjoTN3F6ww1xGjyy/IPzG64Roo0c0yxGjcpeOARx3q6YyGbAziNoEGEMjM7WDrui/Bzr71y3w9yjvI65G3M5601Vt6aZi1Wum2RvFjQ3CqoilgnJ4sXAZApD9BAAwQsxQiD0zOijSIyGQCSbWA0NBAROcbK7fcrHr+44pEuT2rx80bo9p9tsZeOhvs+uMK0p/9hhgZ9pX/do3eNpXky3dBpzfW03bXDFw7Q0XX7uJgsri/R7OvfVa6L94VmqO21ecK+lph9QSwMEFAAAAAgAAAAhAEMqWVOVAAAAcAMAAAoAAABtZW1vcnkuYmlujZIxDsIwDEVtp6QULsHExCE69iDcI957sV6qlAq+UCphOz9Lhqf/HMVEnyjpVljzSEYY3NMCEPn1rYVUJotLjd4O3BJ4T5WXHW9u9PbgHux7z5VXHO9wmO9uchdwydfSFdyrbPuZby68W78X8zHKNac2J4c+MbkELtoXzBVyHfqiPWD0Ne2Lxv8r6Iu4jL5UTfEn+gZQSwMEFAAAAAgAAAAhAKwgWIQtAAAAMwAAABQAAABhZGRpdGlvbmFsX2RhdGEuanNvbqtWyi8tKSgtiU8qzcwpycxTslKoVipITE8tBrFqdRSUEktKijKTSkugIrW1AFBLAwQUAAAACAAAACEAIxC1mksAAABWAAAAGAAAAGV4ZWN1dGlvbl9yZXNvdXJjZXMuanNvbqtWSirNzCnJzIvPzCsuScxLTo1Pzi/NK0ktUrJSqFbKLy0pKC2Jh6oBChnX6igo5cUXl6QWFAO5FmBebmpuflFlfEZ+TipI0KAWAFBLAwQUAAAACAAAACEA2YDFkhYAAAAUAAAADAAAAHZlcnNpb24uanNvbqtWSk7MLMqPL8hMVbJSUDLUM1SqBQBQSwECFAMUAAAACAAAACEARhkeoxgBAABLAgAADQAAAAAAAAAAAAAAgAEAAAAAbWV0YWRhdGEuanNvblBLAQIUAxQAAAAIAAAAIQBDKllTlQAAAHADAAAKAAAAAAAAAAAAAACAAUMBAABtZW1vcnkuYmluUEsBAhQDFAAAAAgAAAAhAKwgWIQtAAAAMwAAABQAAAAAAAAAAAAAAIABAAIAAGFkZGl0aW9uYWxfZGF0YS5qc29uUEsBAhQDFAAAAAgAAAAhACMQtZpLAAAAVgAAABgAAAAAAAAAAAAAAIABXwIAAGV4ZWN1dGlvbl9yZXNvdXJjZXMuanNvblBLAQIUAxQAAAAIAAAAIQDZgMWSFgAAABQAAAAMAAAAAAAAAAAAAACAAeACAAB2ZXJzaW9uLmpzb25QSwUGAAAAAAUABQA1AQAAIAMAAAAA";
pub const CAIRO_FACT: &str = "0x99f8c8b3efce1cb3b53ce44fd5e8339a1299be480cc6e4599d107f69666eb7bb";
pub const LAMBDA_URL: &str = "https://testnet.provingservice.io";
pub const LAMBDA_MAX_PIE_MB: u64 = 20 * 2**20;


pub struct DataAvailabilityWorkere<B: BlockT, C, BE, H> {
    client: Arc<C>,
    substrate_backend: Arc<BE>,
    madara_backend: Arc<mc_db::Backend<B>>,
}

impl<B: BlockT, C, BE, H> Unpin for DataAvailabilityWorkere<B, C, BE, H> {}

#[allow(clippy::too_many_arguments)]
impl<B: BlockT, C, BE, H> DataAvailabilityWorkere<B, C, BE, H> {
    pub fn new(
        client: Arc<C>,
        substrate_backend: Arc<BE>,
        madara_backend: Arc<mc_db::Backend<B>>,
    ) -> Self {
        Self {
            client,
            substrate_backend,
            madara_backend,
        }
    }
}
// Generate the Cairo PIE
// class CairoPie:
//     """
//     A CairoPie is a serializable object containing information about a run of a cairo program.
//     Using the information, one can 'relocate' segments of the run, to make another valid cairo run.
//     For example, this may be used to join a few cairo runs into one, by concatenating respective
//     segments.
//     """

//     metadata: CairoPieMetadata
//     memory: MemoryDict
//     additional_data: Dict[str, Any]
//     execution_resources: ExecutionResources
//     version: Dict[str, str] = field(
//         default_factory=lambda: {"cairo_pie": CURRENT_CAIRO_PIE_VERSION}
//     )

//     METADATA_FILENAME = "metadata.json"
//     MEMORY_FILENAME = "memory.bin"
//     ADDITIONAL_DATA_FILENAME = "additional_data.json"
//     EXECUTION_RESOURCES_FILENAME = "execution_resources.json"
//     VERSION_FILENAME = "version.json"
//     OPTIONAL_FILES = [VERSION_FILENAME]
//     ALL_FILES = [
//         METADATA_FILENAME,
//         MEMORY_FILENAME,
//         ADDITIONAL_DATA_FILENAME,
//         EXECUTION_RESOURCES_FILENAME,
//     ] + OPTIONAL_FILES
//     MAX_SIZE = 1024**3

// ZIP the CairoPIE

// Send zipped CairoPie to SHARP via https://testnet.provingservice.io
//  - /add_job {"cairo_pie": base64.b64encode(cairo_pie.serialize()).decode("ascii")}
// #[tokio::main]
// async fn submit_pie(pie: &str) -> Result {
//     let client = reqwest::Client::builder()
//         .build()?;

//     let mut headers = reqwest::header::HeaderMap::new();
//     headers.insert("Content-Type", "application/json".parse()?);

//     let data = json!({"cairo_pie": pie});


//     let json: serde_json::Value = serde_json::from_str(&data)?;

//     let request = client.request(reqwest::Method::POST, format!("{LAMBDA_URL}/add_job"))
//         .headers(headers)
//         .json(&json);

//     let response = request.send().await?;
//     let body = response.text().await?;

//     println!("{}", body);

//     Ok(())
// }

// Check on job
// - /get_status {"cairo_job_key": job_key}
// #[tokio::main]
// async fn submit_pie(job_key: &str) -> Result {
//     let client = reqwest::Client::builder()
//         .build()?;

//     let mut headers = reqwest::header::HeaderMap::new();
//     headers.insert("Content-Type", "application/json".parse()?);

//     let data = json!({"cairo_job_key": job_key});

//     let json: serde_json::Value = serde_json::from_str(&data)?;

//     let request = client.request(reqwest::Method::POST, format!("{LAMBDA_URL}/get_status"))
//         .headers(headers)
//         .json(&json);

//     let response = request.send().await?;
//     let body = response.text().await?;

//     println!("{}", body);

//     Ok(())
// }

/// Publish data to Ethereum.
/// # Arguments
/// * `sender_id` - The sender id.
/// * `data` - The data to publish.
// async fn publish_data(&self, sender_id: &[u8], data: &[u8]) -> Result<(), &str> {
//     self.check_data(data)?;
//     // Send data to Ethereum.
//     // Check the result
//     // Return the result.
//     abigen!(
//         STARKNET,
//         r#"[
//             function updateState(uint256[] calldata programOutput, uint256 onchainDataHash, uint256 onchainDataSize) external
//         ]"#,
//     );

//     // let provider = Provider::<Http>::try_from(RPC_URL)?;
//     // let client = Arc::new(provider);
//     todo!()
// }

