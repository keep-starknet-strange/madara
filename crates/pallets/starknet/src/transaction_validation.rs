//! Transaction validation logic.
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::CommonAccountFields;
use blockifier::transaction::transactions::ValidatableTransaction;
use frame_support::traits::EnsureOrigin;
use mp_transactions::compute_hash::ComputeTransactionHash;
use starknet_api::transaction::{TransactionSignature, TransactionVersion};

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
    L1Handler { nonce: Felt252Wrapper },
    RegularTxs { sender_address: Felt252Wrapper, transaction_nonce: Felt252Wrapper, sender_nonce: Felt252Wrapper },
}

impl<T: Config> Pallet<T> {
    pub fn validate_unsigned_tx_nonce(
        transaction: &UserOrL1HandlerTransaction,
    ) -> Result<TxPriorityInfo, InvalidTransaction> {
        match transaction {
            UserOrL1HandlerTransaction::User(tx) => {
                let sender_address: ContractAddress = tx.sender_address().into();
                let sender_nonce: Felt252Wrapper = Pallet::<T>::nonce(sender_address).into();
                let transaction_nonce = *tx.nonce();

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

                Ok(TxPriorityInfo::RegularTxs { sender_address: tx.sender_address(), transaction_nonce, sender_nonce })
            }
            UserOrL1HandlerTransaction::L1Handler(tx, _fee) => {
                Self::ensure_l1_message_not_executed(&Nonce(StarkFelt::from(tx.nonce)))?;

                Ok(TxPriorityInfo::L1Handler { nonce: tx.nonce.into() })
            }
        }
    }

    pub fn validate_unsigned_tx(transaction: &UserOrL1HandlerTransaction) -> Result<(), InvalidTransaction> {
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();
        let mut state: BlockifierStateAdapter<T> = BlockifierStateAdapter::<T>::default();
        let mut execution_resources = cairo_vm::vm::runners::cairo_runner::ExecutionResources::default();
        let mut initial_gas = VersionedConstants::latest_constants().tx_initial_gas();

        let _call_info = match transaction {
            UserOrL1HandlerTransaction::User(transaction) => {
                let tx_context = Arc::new(TransactionContext {
                    block_context,
                    tx_info: TransactionInfo::Deprecated(DeprecatedTransactionInfo {
                        common_fields: CommonAccountFields {
                            transaction_hash: transaction.compute_hash::<T::SystemHash>(chain_id, false).into(),
                            version: TransactionVersion(StarkFelt::from(transaction.version())),
                            signature: TransactionSignature(
                                transaction.signature().into_iter().map(|&f| f.into()).collect(),
                            ),
                            nonce: Nonce((*transaction.nonce()).into()),
                            sender_address: ContractAddress(transaction.sender_address().into()),
                            only_query: false,
                        },
                        max_fee: Fee(*transaction.max_fee()),
                    }),
                });

                let validation_result =
                    match transaction {
                        // There is no way to validate it before the account is actuallly deployed
                        UserTransaction::DeployAccount(_) => Ok(None),
                        UserTransaction::Declare(tx, contract_class) => AccountTransaction::Declare(
                            tx.try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), false)
                                .map_err(|_| InvalidTransaction::BadProof)?,
                        )
                        .validate_tx(
                            &mut state,
                            &mut execution_resources,
                            tx_context,
                            &mut initial_gas,
                            false,
                        ),
                        UserTransaction::Invoke(tx) => {
                            AccountTransaction::Invoke(tx.into_executable::<T::SystemHash>(chain_id, false))
                                .validate_tx(&mut state, &mut execution_resources, tx_context, &mut initial_gas, false)
                        }
                    };

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
                    let transaction_nonce = transaction.nonce();
                    let sender_address = transaction.sender_address();
                    if contract_address.0.0 == sender_address.into() && transaction_nonce == &Felt252Wrapper::ONE {
                        Ok(None)
                    } else {
                        validation_result
                    }
                } else {
                    validation_result
                }
            }
            UserOrL1HandlerTransaction::L1Handler(_, fee) => {
                // The tx will fail if no fee have been paid
                if fee.0 == 0 {
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
