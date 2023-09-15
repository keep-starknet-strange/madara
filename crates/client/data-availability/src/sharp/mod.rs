use anyhow::Result;
use serde::Deserialize;
use uuid::Uuid;

#[allow(dead_code)]
pub const LAMBDA_URL: &str = "https://testnet.provingservice.io";
pub const _LAMBDA_MAX_PIE_MB: u64 = 20_971_520;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CairoJobStatus {
    Unknown,
    NotCreated,
    InProgress,
    Processed,
    Onchain,
    Invalid,
    Failed,
}

#[allow(dead_code)]
impl CairoJobStatus {
    fn as_str(&self) -> &'static str {
        match self {
            CairoJobStatus::Unknown => "UNKNOWN",
            CairoJobStatus::NotCreated => "NOT_CREATED",
            CairoJobStatus::InProgress => "IN_PROGRESS",
            CairoJobStatus::Processed => "PROCESSED",
            CairoJobStatus::Onchain => "ONCHAIN",
            CairoJobStatus::Invalid => "INVALID",
            CairoJobStatus::Failed => "FAILED",
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct CairoJobResponse {
    pub cairo_job_key: Uuid,
    pub version: u64,
}

// Send zipped CairoPie to SHARP
// - PIE Submission format base64.b64encode(cairo_pie.serialize()).decode("ascii")
pub fn submit_pie(pie: &str) -> Result<CairoJobResponse> {
    let data = serde_json::json!({ "cairo_pie": pie });
    let data = serde_json::json!({ "action": "add_job", "request": data });
    let _payload: serde_json::Value = serde_json::from_value(data).unwrap();

    // CAREFUL NOT TO OVERWHELM SHARP DUE TO SHORT BLOCK TIMES
    // TODO: uncomment w/ Validity DaMode impl
    // let resp = reqwest::blocking::Client::new().post(LAMBDA_URL).json(&payload).send().unwrap();

    // match resp.status() {
    //     reqwest::StatusCode::OK => Ok(resp.json::<CairoJobResponse>().unwrap()),
    //     _ => Err(String::from("could not submit pie")),
    // }

    Ok(CairoJobResponse { cairo_job_key: Uuid::new_v4(), version: 1_u64 })
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CairoStatusResponse {
    pub status: Option<String>,
    #[serde(rename = "validation_done")]
    pub validation_done: Option<bool>,
    pub version: Option<u64>,
}

// Fetch Cairo Job Status from SHARP
// TODO: function will be needed in Validity DaMode impl
fn _get_status(job_key: &str) -> Result<CairoStatusResponse> {
    let data = serde_json::json!({ "cairo_job_key": job_key });
    let data = serde_json::json!({ "action": "get_status", "request": data });
    let payload: serde_json::Value = serde_json::from_value(data).unwrap();

    let resp = reqwest::blocking::Client::new().post(LAMBDA_URL).json(&payload).send().unwrap();

    match resp.status() {
        reqwest::StatusCode::OK => Ok(resp.json::<CairoStatusResponse>().unwrap()),
        _ => Err(anyhow::anyhow!("could not get job status")),
    }
}
