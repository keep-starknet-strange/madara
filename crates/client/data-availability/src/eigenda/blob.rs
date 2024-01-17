// this is a copy of types in the eigenda disperser API: https://github.com/Layr-Labs/eigenda/blob/master/api/proto/disperser/disperser.proto

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisperseBlobResponse {
    status: BlobStatus,
    #[serde(rename = "requestId")]
    request_id: String
}
impl DisperseBlobResponse {
    pub fn status(&self) -> &BlobStatus {
        &self.status
    }
    pub fn request_id(&self) -> String {
        self.request_id.clone()
    }
}
impl Default for DisperseBlobResponse {
    fn default() -> Self {
        DisperseBlobResponse { status: Default::default(), request_id: Default::default() }
    }
}
impl From<String> for DisperseBlobResponse {
    fn from(value: String) -> Self {
        if let Some(start_index) = value.find('{') {
            let json_str = &value[start_index..];
            println!("{}", &json_str);

            let blob_response: Result<DisperseBlobResponse, serde_json::Error> = serde_json::from_str(json_str);

            match blob_response {
                Ok(response) => {
                    return response
                }
                Err(err) => {
                    println!("{}", &err);
                    return DisperseBlobResponse::default() 
                }
            }
        } else {
            return DisperseBlobResponse::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BlobStatus {
    Processing,
    Confirmed,
    Failed,
    Other(String)
}
impl Default for BlobStatus {
    fn default() -> Self {
        BlobStatus::Other("Default".to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlobStatusResponse {
    status: BlobStatus,
    info: BlobInfo
}
impl BlobStatusResponse {
    pub fn status(&self) -> &BlobStatus {
        &self.status
    }
    pub fn info(&self) -> &BlobInfo {
        &self.info
    }
}
impl Default for BlobStatusResponse {
    fn default() -> Self {
        BlobStatusResponse {
            status: Default::default(),
            info: Default::default()
        }
    }
}
impl From<String> for BlobStatusResponse {
    fn from(value: String) -> Self {
        if let Some(start_index) = value.find('{') {
            let json_str = &value[start_index..];
            let blob_status: Result<BlobStatusResponse, serde_json::Error> = serde_json::from_str(json_str);
            match blob_status {
                Ok(status) => {
                    return status
                }
                Err(_) => {
                    return BlobStatusResponse::default()
                }
            }
        } else {
            return BlobStatusResponse::default()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlobInfo {
    blob_header: BlobHeader,
    blob_verification_proof: BlobVerificationProof,
}
impl BlobInfo {
    pub fn blob_header(&self) -> &BlobHeader {
        &self.blob_header
    }
    pub fn blob_verification_proof(&self) -> &BlobVerificationProof {
        &self.blob_verification_proof
    }
}
impl Default for BlobInfo {
    fn default() -> Self {
        BlobInfo {
            blob_header: Default::default(),
            blob_verification_proof: Default::default()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlobHeader {
    commitment: String,
    data_length: usize,
    blob_quorum_params: Vec<BlobQuorumParams>,
}
impl BlobHeader {
    pub fn commitment(&self) -> &String {
        &self.commitment
    }
    pub fn data_length(&self) -> usize {
        self.data_length
    }
    pub fn blob_quorum_params(&self) -> &Vec<BlobQuorumParams> {
        &self.blob_quorum_params
    }
}
impl Default for BlobHeader {
    fn default() -> Self {
        BlobHeader { 
            commitment: Default::default(),
            data_length: Default::default(),
            blob_quorum_params: Default::default()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BlobVerificationProof {
    batch_id: u128,
    blob_index: u128,
    batch_metadata: BatchMetadata,
    inclusion_proof: String, 
    quorum_indexes: String, 
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BlobQuorumParams {
    adversary_threshold_percentage: usize,
    quorum_threshold_percentage: usize,
    quantization_param: usize,
    encoded_length: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BatchMetadata {
    batch_header: BatchHeader,
    signatory_record_hash: String,
    fee: String,
    confirmation_block_number: u128,
    batch_header_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BatchHeader {
    batch_root: String,
    quorum_numbers: String, 
    quorum_signed_percentages: String,
    reference_block_number: u128
}