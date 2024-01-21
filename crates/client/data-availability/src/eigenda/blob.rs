// this is a copy of types in the eigenda disperser API: https://github.com/Layr-Labs/eigenda/blob/master/api/proto/disperser/disperser.proto

use serde::{Serialize, Deserialize};
use ethers::types::U256;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisperseBlobPayload {
    state_diff: Vec<U256>,
    quorum_id: u32,
    adversary_threshold: u32,
    quorum_threshold: u32,
}
impl DisperseBlobPayload {
    pub(crate) fn new(
        state_diff: Vec<U256>,
        quorum_id: &u32,
        adversary_threshold: &u32, 
        quorum_threshold: &u32,
    ) -> Self {
        DisperseBlobPayload { 
            state_diff,
            quorum_id: *quorum_id,
            adversary_threshold: *adversary_threshold,
            quorum_threshold: *quorum_threshold,
        }
    }
}

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BlobStatus {
    Processing,
    Confirmed,
    Failed,
    Other(String)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlobStatusPayload {
    request_id: String,
}
impl BlobStatusPayload {
    pub(crate) fn new(
        request_id: String,
    ) -> Self {
        BlobStatusPayload { 
            request_id
        }
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