//! Transaction validation logic.
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::errors::{TransactionExecutionError, TransactionPreValidationError};
use blockifier::transaction::transaction_execution::Transaction;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use frame_support::traits::EnsureOrigin;
use mp_transactions::execution::Validate;

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

impl<T: Config> Pallet<T> {
    pub fn pre_validate_unsigned_tx(transaction: &Transaction) -> Result<(), InvalidTransaction> {
        match transaction {
            Transaction::AccountTransaction(transaction) => {
                let mut state = BlockifierStateAdapter::<T>::default();
                let block_context = Self::get_block_context();
                let charge_fee = !<T as Config>::DisableTransactionFee::get();
                let tx_context = Arc::new(block_context.to_tx_context(transaction));
                let string_nonce_checking = false;

                match transaction {
                    AccountTransaction::Declare(transaction) => {
                        Validate::perform_pre_validation_stage(transaction, &mut state, tx_context, string_nonce_checking, charge_fee)
                    }
                    AccountTransaction::DeployAccount(transaction) => {
                        Validate::perform_pre_validation_stage(transaction, &mut state, tx_context, string_nonce_checking, charge_fee)
                    }
                    AccountTransaction::Invoke(transaction) => {
                        Validate::perform_pre_validation_stage(transaction, &mut state, tx_context, string_nonce_checking, charge_fee)
                    }
                }
                // TODO: have more granular error mapping
                .map_err(|_| InvalidTransaction::BadProof)
            }
            Transaction::L1HandlerTransaction(transaction) => {
                Self::ensure_l1_message_not_executed(&transaction.tx.nonce)
            }
        }
    }

    pub fn validate_unsigned_tx(transaction: &Transaction) -> Result<(), InvalidTransaction> {
        let _call_info = match transaction {
            Transaction::AccountTransaction(transaction) => {
                let mut state: BlockifierStateAdapter<T> = BlockifierStateAdapter::<T>::default();
                let block_context = Self::get_block_context();
                let mut inital_gas = block_context.versioned_constants().tx_initial_gas();
                let mut resources = ExecutionResources::default();

                let validation_result = match transaction {
                    AccountTransaction::Declare(tx) => {
                        let tx_context = Arc::new(block_context.to_tx_context(tx));
                        let whitelisted_class_hashes = Self::whitelisted_class_hashes();
                        if !whitelisted_class_hashes.is_empty() && !whitelisted_class_hashes.contains(&tx.class_hash())
                        {
                            return Err(InvalidTransaction::BadProof);
                        }
                        tx.run_validate_entrypoint(&mut state, tx_context, &mut resources, &mut inital_gas, true)
                    }
                    AccountTransaction::DeployAccount(_) => return Ok(()),
                    AccountTransaction::Invoke(tx) => {
                        let tx_context = Arc::new(block_context.to_tx_context(tx));
                        tx.run_validate_entrypoint(&mut state, tx_context, &mut resources, &mut inital_gas, true)
                    }
                };

                // handle the case where we the user sent both its deploy and first tx at the same time
                // we assume that the deploy tx is also in the pool and will therefore be executed before
                // a bit hacky but it is needed in order to be compatible with wallets
                if let Err(TransactionExecutionError::TransactionPreValidationError(
                    TransactionPreValidationError::InvalidNonce { address, account_nonce, incoming_tx_nonce },
                )) = validation_result
                {
                    let sender_address = match transaction {
                        AccountTransaction::Declare(tx) => tx.tx.sender_address(),
                        AccountTransaction::DeployAccount(tx) => tx.contract_address,
                        AccountTransaction::Invoke(tx) => tx.tx.sender_address(),
                    };
                    if address == sender_address
                        && account_nonce == Nonce(StarkFelt::ZERO)
                        && incoming_tx_nonce == Nonce(StarkFelt::ONE)
                    {
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
