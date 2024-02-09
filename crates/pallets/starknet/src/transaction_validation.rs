//! Transaction validation logic.
use blockifier::transaction::errors::TransactionExecutionError;
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
                let transaction_nonce = match tx.nonce() {
                    Some(n) => *n,
                    None => return Ok(TxPriorityInfo::InvokeV0),
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
        let mut execution_resources = ExecutionResources::default();
        let mut initial_gas = blockifier::abi::constants::INITIAL_GAS_COST;

        match transaction {
            UserOrL1HandlerTransaction::User(transaction) => {
                let validation_result =
                    match transaction {
                        // There is no way to validate it before the account is actuallly deployed
                        UserTransaction::DeployAccount(_) => Ok(None),
                        UserTransaction::Declare(tx, contract_class) => tx
                            .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), false)
                            .map_err(|_| InvalidTransaction::BadProof)?
                            .validate_tx(&mut state, &block_context, &mut execution_resources, &mut initial_gas, false),
                        UserTransaction::Invoke(tx) => tx
                            .into_executable::<T::SystemHash>(chain_id, false)
                            .validate_tx(&mut state, &block_context, &mut execution_resources, &mut initial_gas, false),
                    };

                if let Err(TransactionExecutionError::ValidateTransactionError(
                    EntryPointExecutionError::PreExecutionError(PreExecutionError::UninitializedStorageAddress(
                        contract_address,
                    )),
                )) = validation_result
                {
                    let transaction_nonce = transaction.nonce();
                    let sender_address = transaction.sender_address();
                    if contract_address.0.0 == sender_address.into() && transaction_nonce == Some(&Felt252Wrapper::ONE)
                    {
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
