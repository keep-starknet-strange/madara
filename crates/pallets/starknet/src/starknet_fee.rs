use mp_starknet::transaction::types::FeeTransferInformation;
use pallet_transaction_payment::OnChargeTransaction;
use sp_core::U256;
use sp_runtime::traits::{DispatchInfoOf, PostDispatchInfoOf};
use sp_runtime::transaction_validity::TransactionValidityError;

use crate::{Config, Pallet};

pub struct StarknetFee;

impl<T: Config> OnChargeTransaction<T> for StarknetFee {
    /// The underlying integer type in which fees are calculated.
    type Balance = u128;

    /// The underlying integer type of the quantity of tokens.
    type LiquidityInfo = U256;

    /// Before the transaction is executed the payment of the transaction fees
    /// need to be secured.
    ///
    /// Note: The `fee` already includes the `tip`.
    ///
    /// # Arguments
    ///
    /// * `who` - Initiator of the transaction.
    /// * `call` - type of the call.
    /// * `dispatch_info` - dispatch infos.
    /// * `fee` - total fees set by the user.
    /// * `tip` - tip set by the user.
    ///
    /// # Returns
    ///
    /// Fees transferred from the user.
    ///
    /// Error
    ///
    /// Returns an error if any step of the fee transfer fails.
    fn withdraw_fee(
        _who: &T::AccountId,
        _call: &T::RuntimeCall,
        _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        _fee: Self::Balance,
        _tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        Ok(U256::zero())
    }

    /// After the transaction was executed the actual fee can be calculated.
    /// This function should refund any overpaid fees and optionally deposit
    /// the corrected amount.
    ///
    /// Note: The `fee` already includes the `tip`.
    ///
    /// # Arguments
    ///
    /// * `who` - Initiator of the transaction.
    /// * `dispatch_info` - dispatch infos.
    /// * `post_info` - post infos.
    /// * `corrected_fee` - corrected fees after tx execution.
    /// * `tip` - tip set by the user.
    /// * `already_withdrawn` - fees already transferred in the `withdraw_fee` function.
    ///
    /// Error
    ///
    /// Returns an error if any step of the fee transfer refund fails.
    fn correct_and_deposit_fee(
        _who: &T::AccountId,
        _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        _post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        _corrected_fee: Self::Balance,
        tip: Self::Balance,
        _already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        let to = Pallet::<T>::current_block().header().sequencer_address;
        let FeeTransferInformation { actual_fee, payer } = Pallet::<T>::fee_information();
        Pallet::<T>::transfer_fees(payer, to, (actual_fee + tip).as_u128())
    }
}
