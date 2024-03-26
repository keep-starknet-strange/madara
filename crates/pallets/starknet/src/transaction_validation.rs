//! Transaction validation logic.
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionInfoCreator;
use blockifier::transaction::transaction_execution::Transaction;
use blockifier::transaction::transactions::ValidatableTransaction;
use frame_support::traits::EnsureOrigin;

use super::*;

/// Representation of the origin of a Starknet transaction.
/// For now, we still don't know how to represent the origin of a Starknet transaction,
/// given that Starknet has native account abstraction.
/// For now, we just use a dummy origin.
/// See: `https://github.com/keep-starknet-strange/madara/issues/21`
pub enum RawOrigin {
    StarknetTransaction,
}

/// Ensure that the origin is a Starknet transaction.
/// See: `https://github.com/keep-starknet-strange/madara/issues/21`
/// # Arguments
/// * `o` - The origin to check.
/// # Returns
/// * `Result<(), &'static str>` - The result of the check.
pub fn ensure_starknet_transaction<OuterOrigin>(o: OuterOrigin) -> Result<(), &'static str>
where
    OuterOrigin: Into<Result<RawOrigin, OuterOrigin>>,
{
    match o.into() {
        Ok(RawOrigin::StarknetTransaction) => Ok(()),
        _ => Err("bad origin: expected to be an Starknet transaction"),
    }
}

/// Ensure that the origin is a Starknet transaction.
/// See: `https://github.com/keep-starknet-strange/madara/issues/21`
pub struct EnsureStarknetTransaction;
impl<OuterOrigin: Into<Result<RawOrigin, OuterOrigin>> + From<RawOrigin>> EnsureOrigin<OuterOrigin>
    for EnsureStarknetTransaction
{
    type Success = ();

    /// Try to convert the origin into a `RawOrigin::StarknetTransaction`.
    /// # Arguments
    /// * `o` - The origin to check.
    /// # Returns
    /// * `Result<Self::Success, O>` - The result of the check.
    fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
        o.into().map(|o| match o {
            RawOrigin::StarknetTransaction => (),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<OuterOrigin, ()> {
        Ok(OuterOrigin::from(RawOrigin::StarknetTransaction))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TxPriorityInfo {
    InvokeV0,
    L1Handler { nonce: Nonce },
    RegularTxs { sender_address: ContractAddress, transaction_nonce: Nonce, sender_nonce: Nonce },
}

impl<T: Config> Pallet<T> {
    pub fn validate_unsigned_tx_nonce(transaction: &Transaction) -> Result<TxPriorityInfo, InvalidTransaction> {
        match transaction {
            Transaction::AccountTransaction(tx) => {
                let sender_address = match tx {
                    AccountTransaction::Declare(tx) => match &tx.tx {
                        starknet_api::transaction::DeclareTransaction::V0(tx) => tx.sender_address,
                        starknet_api::transaction::DeclareTransaction::V1(tx) => tx.sender_address,
                        starknet_api::transaction::DeclareTransaction::V2(tx) => tx.sender_address,
                        starknet_api::transaction::DeclareTransaction::V3(tx) => tx.sender_address,
                    },
                    AccountTransaction::DeployAccount(tx) => tx.contract_address,
                    AccountTransaction::Invoke(tx) => match &tx.tx {
                        starknet_api::transaction::InvokeTransaction::V0(tx) => tx.contract_address,
                        starknet_api::transaction::InvokeTransaction::V1(tx) => tx.sender_address,
                        starknet_api::transaction::InvokeTransaction::V3(tx) => tx.sender_address,
                    },
                };
                let sender_nonce = Pallet::<T>::nonce(sender_address);
                let transaction_nonce = match tx {
                    AccountTransaction::Declare(tx) => match &tx.tx {
                        starknet_api::transaction::DeclareTransaction::V0(tx) => tx.nonce,
                        starknet_api::transaction::DeclareTransaction::V1(tx) => tx.nonce,
                        starknet_api::transaction::DeclareTransaction::V2(tx) => tx.nonce,
                        starknet_api::transaction::DeclareTransaction::V3(tx) => tx.nonce,
                    },
                    AccountTransaction::DeployAccount(tx) => match &tx.tx {
                        starknet_api::transaction::DeployAccountTransaction::V1(tx) => tx.nonce,
                        starknet_api::transaction::DeployAccountTransaction::V3(tx) => tx.nonce,
                    },
                    AccountTransaction::Invoke(tx) => match &tx.tx {
                        starknet_api::transaction::InvokeTransaction::V0(_) => return Ok(TxPriorityInfo::InvokeV0),
                        starknet_api::transaction::InvokeTransaction::V1(tx) => tx.nonce,
                        starknet_api::transaction::InvokeTransaction::V3(tx) => tx.nonce,
                    },
                };

                // Reject transaction with an already used Nonce
                if sender_nonce > transaction_nonce {
                    Err(InvalidTransaction::Stale)?;
                }

                // A transaction with a nonce higher than the expected nonce is placed in
                // the future queue of the transaction pool.
                if sender_nonce < transaction_nonce {
                    log::debug!(
                        "Nonce is too high. Expected: {:?}, got: {:?}. This transaction will be placed in the \
                         transaction pool and executed in the future when the nonce is reached.",
                        sender_nonce,
                        transaction_nonce
                    );
                }

                let sender_address = match tx {
                    AccountTransaction::Declare(tx) => tx.tx.sender_address(),
                    AccountTransaction::DeployAccount(tx) => tx.contract_address,
                    AccountTransaction::Invoke(tx) => tx.tx.sender_address(),
                };

                Ok(TxPriorityInfo::RegularTxs { sender_address, transaction_nonce, sender_nonce })
            }
            Transaction::L1HandlerTransaction(tx) => {
                Self::ensure_l1_message_not_executed(&tx.tx.nonce)?;

                Ok(TxPriorityInfo::L1Handler { nonce: tx.tx.nonce })
            }
        }
    }

    pub fn validate_unsigned_tx(transaction: &Transaction) -> Result<(), InvalidTransaction> {
        let mut state: BlockifierStateAdapter<T> = BlockifierStateAdapter::<T>::default();
        let mut execution_resources = cairo_vm::vm::runners::cairo_runner::ExecutionResources::default();
        let mut initial_gas = VersionedConstants::latest_constants().tx_initial_gas();

        let _call_info = match transaction {
            Transaction::AccountTransaction(transaction) => {
                let tx_context = Arc::new(TransactionContext {
                    block_context: Self::get_block_context(),
                    tx_info: transaction.create_tx_info(),
                });

                let validation_result =
                    transaction.validate_tx(&mut state, &mut execution_resources, tx_context, &mut initial_gas, false);

                // handle the case where we the user sent both its deploy and first tx at the same time
                // we assume that the deploy tx is also in the pool and will therefore be executed before
                // a bit hacky but it is needed in order to be compatible with wallets
                if let Err(TransactionExecutionError::ValidateTransactionError {
                    error:
                        EntryPointExecutionError::PreExecutionError(PreExecutionError::UninitializedStorageAddress(
                            contract_address,
                        )),
                    storage_address: _,
                    selector: _,
                }) = validation_result
                {
                    let transaction_nonce = match transaction {
                        AccountTransaction::Declare(tx) => tx.tx.nonce(),
                        AccountTransaction::DeployAccount(tx) => tx.tx.nonce(),
                        AccountTransaction::Invoke(tx) => tx.tx.nonce(),
                    };
                    let sender_address = match transaction {
                        AccountTransaction::Declare(tx) => tx.tx.sender_address(),
                        AccountTransaction::DeployAccount(tx) => tx.contract_address,
                        AccountTransaction::Invoke(tx) => tx.tx.sender_address(),
                    };
                    if contract_address == sender_address && transaction_nonce == Nonce(StarkFelt::ONE) {
                        Ok(None)
                    } else {
                        validation_result
                    }
                } else {
                    validation_result
                }
            }
            Transaction::L1HandlerTransaction(tx) => {
                // The tx will fail if no fee have been paid
                if tx.paid_fee_on_l1 == Fee(0) {
                    return Err(InvalidTransaction::Payment);
                }

                Ok(None)
            }
        }
        .map_err(|_| InvalidTransaction::BadProof)?;

        Ok(())
    }

    pub fn ensure_l1_message_not_executed(nonce: &Nonce) -> Result<(), InvalidTransaction> {
        if L1Messages::<T>::get().contains(nonce) { Err(InvalidTransaction::Stale) } else { Ok(()) }
    }
}
