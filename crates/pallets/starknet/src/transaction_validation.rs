//! Transaction validation logic.
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
    pub fn validate_usigned_tx_nonce(
        transaction: &UserAndL1HandlerTransaction,
    ) -> Result<(Felt252Wrapper, Felt252Wrapper, Felt252Wrapper), InvalidTransaction> {
        let (sender_address, sender_nonce, transaction_nonce) = match transaction {
            UserAndL1HandlerTransaction::User(tx) => {
                let sender_address: ContractAddress = tx.sender_address().into();
                let sender_nonce: Felt252Wrapper = Pallet::<T>::nonce(sender_address).into();
                let transaction_nonce = tx.nonce();
                // Reject transaction with an already used Nonce
                if sender_nonce > *transaction_nonce {
                    Err(InvalidTransaction::Stale)?;
                }

                // A transaction with a nonce higher than the expected nonce is placed in
                // the future queue of the transaction pool.
                if sender_nonce < *transaction_nonce {
                    log!(
                        info,
                        "Nonce is too high. Expected: {:?}, got: {:?}. This transaction will be placed in the \
                         transaction pool and executed in the future when the nonce is reached.",
                        sender_nonce,
                        transaction_nonce
                    );
                }

                (tx.sender_address(), sender_nonce, *transaction_nonce)
            }
            UserAndL1HandlerTransaction::L1Handler(tx, _fee) => {
                let sender_address = ContractAddress::default();
                let sender_nonce = Pallet::<T>::nonce(sender_address).into();
                let nonce = tx.nonce.into();

                Self::ensure_l1_message_not_executed(&nonce)?;
                (sender_address.into(), sender_nonce, nonce)
            }
        };

        Ok((sender_address, sender_nonce, transaction_nonce))
    }

    pub fn validate_unsigned_tx(transaction: &UserAndL1HandlerTransaction) -> Result<(), InvalidTransaction> {
        let chain_id = Self::chain_id();
        let block_context = Self::get_block_context();
        let mut state: BlockifierStateAdapter<T> = BlockifierStateAdapter::<T>::default();
        let mut execution_resources = ExecutionResources::default();
        let mut initial_gas = blockifier::abi::constants::INITIAL_GAS_COST;

        match transaction {
            UserAndL1HandlerTransaction::User(transaction) => {
                match transaction {
                    // There is no way to validate it before the account is actuallly deployed
                    UserTransaction::DeployAccount(_) => Ok(None),
                    UserTransaction::Declare(tx, contract_class) => tx
                        .try_into_executable::<T::SystemHash>(chain_id, contract_class.clone(), false)
                        .map_err(|_| InvalidTransaction::BadProof)?
                        .validate_tx(&mut state, &block_context, &mut execution_resources, &mut initial_gas, false),
                    UserTransaction::Invoke(tx) => tx.into_executable::<T::SystemHash>(chain_id, false).validate_tx(
                        &mut state,
                        &block_context,
                        &mut execution_resources,
                        &mut initial_gas,
                        false,
                    ),
                }
            }
            UserAndL1HandlerTransaction::L1Handler(transaction, fee) => transaction
                .into_executable::<T::SystemHash>(chain_id, *fee, false)
                .validate_tx(&mut state, &block_context, &mut execution_resources, &mut initial_gas, false),
        }
        .map_or_else(|_error| Err(InvalidTransaction::BadProof), |_res| Ok(()))
    }

    pub fn ensure_l1_message_not_executed(nonce: &Felt252Wrapper) -> Result<(), InvalidTransaction> {
        match L1Messages::<T>::contains_key(nonce) {
            true => Err(InvalidTransaction::Stale),
            false => Ok(()),
        }
    }
}
