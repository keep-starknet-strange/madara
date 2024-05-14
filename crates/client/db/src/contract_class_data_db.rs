use std::sync::Arc;

use mp_transactions::{ContractClassData, V0ContractClassData, V1ContractClassData};
use parity_scale_codec::Encode;
use sp_database::Database;
use starknet_api::core::ClassHash;

use crate::{DbError, DbHash};

/// Allow interaction with the mapping db
pub struct ContractClassDataDb {
    pub(crate) db: Arc<dyn Database<DbHash>>,
}

impl ContractClassDataDb {
    pub fn register_pending_v0_contract_class_data(
        &self,
        class_hash: ClassHash,
        data: V0ContractClassData,
    ) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        let data_as_json_vec = {
            // Forced to convert to json because `LegacyContractAbiEntry` doesn't impl scale-codec
            let mut data_as_json_vec = serde_json::to_vec(&data)?;
            // Push the contract class version at the end
            data_as_json_vec.push(0);

            data_as_json_vec
        };
        transaction.set(crate::columns::PENDING_CONTRACT_CLASS_DATA, &class_hash.encode(), &data_as_json_vec);

        self.db.commit(transaction)?;

        Ok(())
    }

    pub fn register_pending_v1_contract_class_data(
        &self,
        class_hash: ClassHash,
        data: V1ContractClassData,
    ) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();
        let data_as_json_vec = {
            // Forced to convert to json because `FlattenedSierraClass` doesn't impl scale-codec
            let mut data_as_json_vec = serde_json::to_vec(&data)?;
            // Push the contract class version at the end
            data_as_json_vec.push(1);

            data_as_json_vec
        };

        transaction.set(crate::columns::PENDING_CONTRACT_CLASS_DATA, &class_hash.encode(), &data_as_json_vec);

        self.db.commit(transaction)?;

        Ok(())
    }

    pub fn read_contract_class_data(&self, class_hash: ClassHash) -> Result<Option<ContractClassData>, DbError> {
        let raw_data = match self.db.get(crate::columns::CONTRACT_CLASS_DATA, &class_hash.encode()) {
            Some(raw) => raw,
            None => return Ok(None),
        };

        let contract_class_data = match raw_data
            .last()
            .ok_or_else(|| DbError::CorruptedValue(crate::columns::CONTRACT_CLASS_DATA, class_hash.to_string()))?
        {
            0 => ContractClassData::V0(serde_json::from_slice(&raw_data[..raw_data.len() - 1])?),
            1 => ContractClassData::V1(serde_json::from_slice(&raw_data[..raw_data.len() - 1])?),
            _ => return Err(DbError::CorruptedValue(crate::columns::CONTRACT_CLASS_DATA, class_hash.to_string())),
        };

        Ok(Some(contract_class_data))
    }

    pub fn remove_pending_contract_class_data(&self, class_hash: ClassHash) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        transaction.remove(crate::columns::PENDING_CONTRACT_CLASS_DATA, &class_hash.encode());

        self.db.commit(transaction)?;

        Ok(())
    }

    pub fn persist_pending_contract_class_data(&self, class_hash: ClassHash) -> Result<(), DbError> {
        let encoded_class_hash = class_hash.encode();

        let encoded_class_hash_data =
            self.db.get(crate::columns::PENDING_CONTRACT_CLASS_DATA, &encoded_class_hash).ok_or_else(|| {
                DbError::ValueNotInitialized(crate::columns::PENDING_CONTRACT_CLASS_DATA, class_hash.to_string())
            })?;

        let mut transaction = sp_database::Transaction::new();

        transaction.remove(crate::columns::PENDING_CONTRACT_CLASS_DATA, &encoded_class_hash);
        transaction.set(crate::columns::CONTRACT_CLASS_DATA, &encoded_class_hash, &encoded_class_hash_data);

        self.db.commit(transaction)?;

        Ok(())
    }
}
