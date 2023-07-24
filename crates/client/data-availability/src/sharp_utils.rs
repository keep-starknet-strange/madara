use serde::Deserialize;
use uuid::Uuid;

pub const TEST_CAIRO_PIE_BASE64: &str = "UEsDBBQAAAAIAAAAIQBGGR6jGAEAAEsCAAANAAAAbWV0YWRhdGEuanNvbnWR3WqDQBCFX0W8zsX87M7O9lVCCDbZBqEa0RVCgu/e0aaYgL1bznwz5xx9lF1/vfRVU34Uj/Jc5coeeyfIHr0gEaojz87tCgTYFR5V1YMTYIwRgjoTN3F6ww1xGjyy/IPzG64Roo0c0yxGjcpeOARx3q6YyGbAziNoEGEMjM7WDrui/Bzr71y3w9yjvI65G3M5601Vt6aZi1Wum2RvFjQ3CqoilgnJ4sXAZApD9BAAwQsxQiD0zOijSIyGQCSbWA0NBAROcbK7fcrHr+44pEuT2rx80bo9p9tsZeOhvs+uMK0p/9hhgZ9pX/do3eNpXky3dBpzfW03bXDFw7Q0XX7uJgsri/R7OvfVa6L94VmqO21ecK+lph9QSwMEFAAAAAgAAAAhAEMqWVOVAAAAcAMAAAoAAABtZW1vcnkuYmlujZIxDsIwDEVtp6QULsHExCE69iDcI957sV6qlAq+UCphOz9Lhqf/HMVEnyjpVljzSEYY3NMCEPn1rYVUJotLjd4O3BJ4T5WXHW9u9PbgHux7z5VXHO9wmO9uchdwydfSFdyrbPuZby68W78X8zHKNac2J4c+MbkELtoXzBVyHfqiPWD0Ne2Lxv8r6Iu4jL5UTfEn+gZQSwMEFAAAAAgAAAAhAKwgWIQtAAAAMwAAABQAAABhZGRpdGlvbmFsX2RhdGEuanNvbqtWyi8tKSgtiU8qzcwpycxTslKoVipITE8tBrFqdRSUEktKijKTSkugIrW1AFBLAwQUAAAACAAAACEAIxC1mksAAABWAAAAGAAAAGV4ZWN1dGlvbl9yZXNvdXJjZXMuanNvbqtWSirNzCnJzIvPzCsuScxLTo1Pzi/NK0ktUrJSqFbKLy0pKC2Jh6oBChnX6igo5cUXl6QWFAO5FmBebmpuflFlfEZ+TipI0KAWAFBLAwQUAAAACAAAACEA2YDFkhYAAAAUAAAADAAAAHZlcnNpb24uanNvbqtWSk7MLMqPL8hMVbJSUDLUM1SqBQBQSwECFAMUAAAACAAAACEARhkeoxgBAABLAgAADQAAAAAAAAAAAAAAgAEAAAAAbWV0YWRhdGEuanNvblBLAQIUAxQAAAAIAAAAIQBDKllTlQAAAHADAAAKAAAAAAAAAAAAAACAAUMBAABtZW1vcnkuYmluUEsBAhQDFAAAAAgAAAAhAKwgWIQtAAAAMwAAABQAAAAAAAAAAAAAAIABAAIAAGFkZGl0aW9uYWxfZGF0YS5qc29uUEsBAhQDFAAAAAgAAAAhACMQtZpLAAAAVgAAABgAAAAAAAAAAAAAAIABXwIAAGV4ZWN1dGlvbl9yZXNvdXJjZXMuanNvblBLAQIUAxQAAAAIAAAAIQDZgMWSFgAAABQAAAAMAAAAAAAAAAAAAACAAeACAAB2ZXJzaW9uLmpzb25QSwUGAAAAAAUABQA1AQAAIAMAAAAA";
pub const TEST_JOB_ID: &str = "61a99af0-dcbb-47c9-8350-a08e488af073";
pub const LAMBDA_URL: &str = "https://testnet.provingservice.io";
pub const _LAMBDA_MAX_PIE_MB: u64 = 20_971_520;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct CairoJobResponse {
    pub cairo_job_key: String,
    pub version: u64,
}

// Send zipped CairoPie to SHARP
// - PIE Submission format base64.b64encode(cairo_pie.serialize()).decode("ascii")
pub fn submit_pie(pie: &str) -> Result<CairoJobResponse, String> {
    let data = serde_json::json!({ "cairo_pie": pie });
    let data = serde_json::json!({ "action": "add_job", "request": data });
    let _payload: serde_json::Value = serde_json::from_value(data).unwrap();

    // CAREFUL NOT TO OVERWHELM SHARP DUE TO SHORT BLOCK TIMES
    // let resp = reqwest::blocking::Client::new().post(LAMBDA_URL).json(&payload).send().unwrap();

    // match resp.status() {
    //     reqwest::StatusCode::OK => Ok(resp.json::<CairoJobResponse>().unwrap()),
    //     _ => Err(String::from("could not submit pie")),
    // }

    Ok(CairoJobResponse { cairo_job_key: Uuid::new_v4().to_string(), version: 1_u64 })
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
