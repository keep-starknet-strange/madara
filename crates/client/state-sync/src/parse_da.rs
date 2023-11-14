use ethereum_types::H256;

pub struct ContractUpdate {
    contract: H256,
    da_word: u64,
    class_hash: Option<H256>,
    nonce: Option<u64>,
    storage_updates: Option<u64>,
    keys_and_values: Vec<(H256, H256)>,
}

pub struct ParsedDAData {
    pub contract_updates: Vec<ContractUpdate>,
    pub declared_classes: Option<u64>,
}

// const NONCE_BASE: u128 = 2u128.pow(64);

//const NONCE_BASE: u64 = 1 << 64;

const NONCE_BASE: u64 = std::u64::MAX;

pub fn start_parse_da(fact_input_sn_output: Vec<u64>) -> ParsedDAData {
	println!("== Starting parse_da ==");

    let mut result = ParsedDAData {
        contract_updates: Vec::new(),
        declared_classes: None,
    };

    let mut idx = 0;
    while idx < fact_input_sn_output.len() {
        let num_storage_updates = fact_input_sn_output[idx];
        idx += 1;

        let mut contract_updates = Vec::new();
        for _ in 0..num_storage_updates {
            let contract = H256::from_low_u64_be(fact_input_sn_output[idx]);
            idx += 1;
            let da_word = fact_input_sn_output[idx];
            idx += 1;

            let (class_hash, nonce, storage_updates) = if da_word > NONCE_BASE {
                let da_word_binary = format!("{:064b}", da_word);
                let word_length = da_word_binary.len();
                if word_length == 129 {
                    let class_hash = H256::from_low_u64_be(fact_input_sn_output[idx]);
                    idx += 1;
                    let nonce = u64::from_str_radix(&da_word_binary[1..65], 2).unwrap();
                    let storage_updates = u64::from_str_radix(&da_word_binary[65..], 2).unwrap();
                    (Some(class_hash), Some(nonce), Some(storage_updates))
                } else if word_length > 64 {
                    let nonce = u64::from_str_radix(&da_word_binary[0..(word_length - 64)], 2).unwrap();
                    let storage_updates = u64::from_str_radix(&da_word_binary[(word_length - 64)..], 2).unwrap();
                    (None, Some(nonce), Some(storage_updates))
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, Some(da_word as u64))
            };

            let mut keys_and_values = Vec::new();
            for _ in 0..storage_updates.unwrap_or(0) {
                let key = H256::from_low_u64_be(fact_input_sn_output[idx]);
                idx += 1;
                let value = H256::from_low_u64_be(fact_input_sn_output[idx]);
                idx += 1;
                keys_and_values.push((key, value));
            }

            contract_updates.push(ContractUpdate {
                contract,
                da_word,
                class_hash,
                nonce,
                storage_updates,
                keys_and_values,
            });
        }

        result.contract_updates = contract_updates;
        if idx < fact_input_sn_output.len() {
            result.declared_classes = Some(fact_input_sn_output[idx]);
            idx += 1;
        }
    }

    result
}





