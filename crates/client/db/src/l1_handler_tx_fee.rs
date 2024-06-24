use std::sync::Arc;

use parity_scale_codec::Encode;
use sp_database::Database;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Fee;

use crate::{DbError, DbHash};

pub struct L1HandlerTxFeeDb {
    pub(crate) db: Arc<dyn Database<DbHash>>,
}

impl L1HandlerTxFeeDb {
    /// Store the fee paid on l1 for a specific L1Handler transaction
    pub fn store_fee_paid_for_l1_handler_tx(&self, tx_hash: StarkFelt, fee: Fee) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::L1_HANDLER_PAID_FEE, &tx_hash.encode(), &fee.0.to_le_bytes());

        self.db.commit(transaction)?;

        Ok(())
    }

    /// Return the stored fee paid on l1 for a specific L1Handler transaction
    pub fn get_fee_paid_for_l1_handler_tx(&self, tx_hash: StarkFelt) -> Result<Option<Fee>, DbError> {
        let opt_fee = self.db.get(crate::columns::L1_HANDLER_PAID_FEE, &tx_hash.encode()).map(|raw| {
            let mut buff = [0u8; 16];

            buff.copy_from_slice(&raw);
            let fee = u128::from_le_bytes(buff);

            Fee(fee)
        });

        Ok(opt_fee)
    }
}
