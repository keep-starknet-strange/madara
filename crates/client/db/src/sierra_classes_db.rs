use std::sync::Arc;

use parity_scale_codec::{Decode, Encode};
use sp_database::Database;
use starknet_api::api_core::ClassHash;
use starknet_api::state::ContractClass;

use crate::{DbError, DbHash};

/// Allow interaction with the sierra classes db
pub struct SierraClassesDb {
    pub(crate) db: Arc<dyn Database<DbHash>>,
}

impl SierraClassesDb {
    pub fn store_sierra_class(&self, class_hash: ClassHash, class: ContractClass) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::SIERRA_CONTRACT_CLASSES, &class_hash.encode(), &class.encode());

        self.db.commit(transaction)?;

        Ok(())
    }

    pub fn get_sierra_class(&self, class_hash: ClassHash) -> Result<Option<ContractClass>, DbError> {
        let opt_contract_class = self
            .db
            .get(crate::columns::SIERRA_CONTRACT_CLASSES, &class_hash.encode())
            .map(|raw| ContractClass::decode(&mut &raw[..]))
            .transpose()?;

        Ok(opt_contract_class)
    }
}
