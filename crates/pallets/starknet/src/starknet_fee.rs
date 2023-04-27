use alloc::string::ToString;

use blockifier::abi::abi_utils;
use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::{ExecutionContext, ExecutionResources};
use blockifier::transaction::constants::TRANSFER_ENTRY_POINT_NAME;
use blockifier::transaction::objects::AccountTransactionContext;
use mp_starknet::execution::ContractAddressWrapper;
use mp_starknet::transaction::types::FeeTransferInformation;
use pallet_transaction_payment::OnChargeTransaction;
use sp_core::U256;
use sp_runtime::traits::{DispatchInfoOf, PostDispatchInfoOf};
use sp_runtime::transaction_validity::InvalidTransaction::Payment;
use sp_runtime::transaction_validity::TransactionValidityError;
use sp_runtime::transaction_validity::UnknownTransaction::Custom;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkFelt;
use starknet_api::stdlib::collections::HashMap;
use starknet_api::transaction::Calldata;

use super::log;
use crate::blockifier_state_adapter::BlockifierStateAdapter;
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

impl<T: Config> Pallet<T> {
    /// Helper function that will transfer some fee token.
    ///
    /// # Arguments
    ///
    /// * `from` - the sender of the tokens
    /// * `to` - recipient of the tokens
    /// * `amount` - amount of the tokens
    ///
    /// # Error
    ///
    /// Returns an error if a step of the transfer fails
    pub fn transfer_fees(
        from: ContractAddressWrapper,
        to: ContractAddressWrapper,
        amount: <StarknetFee as OnChargeTransaction<T>>::Balance,
    ) -> Result<(), TransactionValidityError> {
        // Get current block.
        let block = Pallet::<T>::current_block();
        let fee_token_address =
            ContractAddress::try_from(StarkFelt::new(Pallet::<T>::fee_token_address()).map_err(|_| {
                log!(error, "Couldn't convert fee_token_address to StarkFelt");
                TransactionValidityError::Unknown(Custom(0_u8))
            })?)
            .map_err(|_| {
                log!(error, "Couldn't convert StarkFelt to ContractAddress");
                TransactionValidityError::Unknown(Custom(1_u8))
            })?;
        // Create fee transfer transaction.
        let fee_transfer_call = blockifier::execution::entry_point::CallEntryPoint {
            class_hash: None,
            entry_point_type: EntryPointType::External,
            entry_point_selector: abi_utils::selector_from_name(TRANSFER_ENTRY_POINT_NAME),
            calldata: starknet_api::calldata![
                StarkFelt::new(to).map_err(|_| {
                    log!(error, "Couldn't convert sequencer address to StarkFelt");
                    TransactionValidityError::Unknown(Custom(0_u8))
                })?, // Recipient.
                StarkFelt::new([[0_u8; 16], amount.to_be_bytes()].concat()[..32].try_into().map_err(|_| {
                    log!(error, "Couldn't convert fees to StarkFelt");
                    TransactionValidityError::Unknown(Custom(0_u8))
                })?)
                .map_err(|_| {
                    log!(error, "Couldn't convert fees to StarkFelt");
                    TransactionValidityError::Unknown(Custom(0_u8))
                })?, // low
                StarkFelt::default() // high
            ],
            storage_address: fee_token_address,
            caller_address: ContractAddress::try_from(StarkFelt::new(from).map_err(|_| {
                log!(error, "Couldn't convert StarkFelt to ContractAddress");
                TransactionValidityError::Unknown(Custom(1_u8))
            })?)
            .map_err(|_| {
                log!(error, "Couldn't convert StarkFelt to ContractAddress");
                TransactionValidityError::Unknown(Custom(1_u8))
            })?,
            call_type: blockifier::execution::entry_point::CallType::Call,
        };
        // FIXME #245
        let mut execution_context = ExecutionContext::default(); // TODO: check if it needs a real value.
        let account_ctx = AccountTransactionContext::default(); // TODO: check if it needs a real value.
        // FIXME #256
        let block_ctx = BlockContext {
            chain_id: ChainId("SN_GOERLI".to_string()), // TODO: Make it configurable ?
            block_number: BlockNumber(block.header().block_number.as_u64()),
            block_timestamp: BlockTimestamp(block.header().block_timestamp),
            sequencer_address: ContractAddress::try_from(StarkFelt::new(block.header().sequencer_address).map_err(
                |_| {
                    log!(error, "Couldn't convert sequencer address to StarkFelt");
                    TransactionValidityError::Unknown(Custom(0_u8))
                },
            )?)
            .map_err(|_| {
                log!(error, "Couldn't convert StarkFelt to ContractAddress");
                TransactionValidityError::Unknown(Custom(1_u8))
            })?,
            cairo_resource_fee_weights: HashMap::default(), // TODO: Use real weights
            fee_token_address,
            invoke_tx_max_n_steps: 1000000, // TODO: Make it configurable
            validate_max_n_steps: 1000000,  // TODO: Make it configurable
            gas_price: 0,                   // TODO: Use block gas price
        };
        match fee_transfer_call.execute(
            &mut BlockifierStateAdapter::<T>::default(),
            &mut ExecutionResources::default(),
            &mut execution_context,
            &block_ctx,
            &account_ctx,
        ) {
            Ok(mut v) => {
                log!(trace, "Fees executed successfully: {:?}", v.execution.events);
                Self::emit_events(&mut v).map_err(|_| TransactionValidityError::Unknown(Custom(4_u8)))?;
            }
            Err(e) => {
                log!(error, "Fees execution failed: {:?}", e);
                return Err(TransactionValidityError::Invalid(Payment));
            }
        }

        Ok(())
    }
}
