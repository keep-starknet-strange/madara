use std::vec::Vec;

use mp_felt::Felt252Wrapper;
use starknet_crypto::FieldElement;

fn cast_vec_of_felt_252_wrappers(data: Vec<Felt252Wrapper>) -> Vec<FieldElement> {
    // Non-copy but less dangerous than transmute
    // https://doc.rust-lang.org/std/mem/fn.transmute.html#alternatives

    // Unsafe code but all invariants are checked:

    // 1. ptr must have been allocated using the global allocator -> data is allocated with the Global
    //    allocator.
    // 2. T needs to have the same alignment as what ptr was allocated with -> Felt252Wrapper uses
    //    transparent representation of the inner type.
    // 3. The allocated size in bytes needs to be the same as the pointer -> As FieldElement and
    //    Felt252Wrapper have the same size, and capacity is taken directly from the data Vector, we
    //    will have the same allocated byte size.
    // 4. Length needs to be less than or equal to capacity -> data.len() is always less than or equal
    //    to data.capacity()
    // 5. The first length values must be properly initialized values of type T -> ok since we use data
    //    which was correctly allocated
    // 6. capacity needs to be the capacity that the pointer was allocated with -> data.as_mut_ptr()
    //    returns a pointer to memory having at least capacity initialized memory
    // 7. The allocated size in bytes must be no larger than isize::MAX -> data.capacity() will never be
    //    bigger than isize::MAX (https://doc.rust-lang.org/std/vec/struct.Vec.html#panics-7)
    let mut data = core::mem::ManuallyDrop::new(data);
    unsafe { alloc::vec::Vec::from_raw_parts(data.as_mut_ptr() as *mut FieldElement, data.len(), data.capacity()) }
}

pub fn to_starknet_core_tx(
    tx: super::Transaction,
    transaction_hash: FieldElement,
) -> starknet_core::types::Transaction {
    match tx {
        super::Transaction::Declare(tx) => {
            let tx = match tx {
                super::DeclareTransaction::V0(super::DeclareTransactionV0 {
                    max_fee,
                    signature,
                    nonce: _,
                    class_hash,
                    sender_address,
                }) => starknet_core::types::DeclareTransaction::V0(starknet_core::types::DeclareTransactionV0 {
                    transaction_hash,
                    max_fee: max_fee.into(),
                    signature: cast_vec_of_felt_252_wrappers(signature),
                    class_hash: class_hash.into(),
                    sender_address: sender_address.into(),
                }),
                super::DeclareTransaction::V1(super::DeclareTransactionV1 {
                    max_fee,
                    signature,
                    nonce,
                    class_hash,
                    sender_address,
                    ..
                }) => starknet_core::types::DeclareTransaction::V1(starknet_core::types::DeclareTransactionV1 {
                    transaction_hash,
                    max_fee: max_fee.into(),
                    signature: cast_vec_of_felt_252_wrappers(signature),
                    nonce: nonce.into(),
                    class_hash: class_hash.into(),
                    sender_address: sender_address.into(),
                }),
                super::DeclareTransaction::V2(super::DeclareTransactionV2 {
                    max_fee,
                    signature,
                    nonce,
                    class_hash,
                    sender_address,
                    compiled_class_hash,
                    ..
                }) => starknet_core::types::DeclareTransaction::V2(starknet_core::types::DeclareTransactionV2 {
                    transaction_hash,
                    max_fee: max_fee.into(),
                    signature: cast_vec_of_felt_252_wrappers(signature),
                    nonce: nonce.into(),
                    class_hash: class_hash.into(),
                    sender_address: sender_address.into(),
                    compiled_class_hash: compiled_class_hash.into(),
                }),
            };

            starknet_core::types::Transaction::Declare(tx)
        }
        super::Transaction::DeployAccount(tx) => {
            let tx = starknet_core::types::DeployAccountTransaction {
                transaction_hash,
                max_fee: tx.max_fee.into(),
                signature: cast_vec_of_felt_252_wrappers(tx.signature),
                nonce: tx.nonce.into(),
                contract_address_salt: tx.contract_address_salt.into(),
                constructor_calldata: cast_vec_of_felt_252_wrappers(tx.constructor_calldata),
                class_hash: tx.class_hash.into(),
            };

            starknet_core::types::Transaction::DeployAccount(tx)
        }
        super::Transaction::Invoke(tx) => {
            let tx = match tx {
                super::InvokeTransaction::V0(super::InvokeTransactionV0 {
                    max_fee,
                    signature,
                    contract_address,
                    entry_point_selector,
                    calldata,
                }) => starknet_core::types::InvokeTransaction::V0(starknet_core::types::InvokeTransactionV0 {
                    transaction_hash,
                    max_fee: max_fee.into(),
                    signature: cast_vec_of_felt_252_wrappers(signature),
                    contract_address: contract_address.into(),
                    entry_point_selector: entry_point_selector.into(),
                    calldata: cast_vec_of_felt_252_wrappers(calldata),
                }),
                super::InvokeTransaction::V1(super::InvokeTransactionV1 {
                    max_fee,
                    signature,
                    nonce,
                    sender_address,
                    calldata,
                    ..
                }) => starknet_core::types::InvokeTransaction::V1(starknet_core::types::InvokeTransactionV1 {
                    transaction_hash,
                    max_fee: max_fee.into(),
                    signature: cast_vec_of_felt_252_wrappers(signature),
                    nonce: nonce.into(),
                    sender_address: sender_address.into(),
                    calldata: cast_vec_of_felt_252_wrappers(calldata),
                }),
            };

            starknet_core::types::Transaction::Invoke(tx)
        }
        super::Transaction::L1Handler(tx) => {
            let tx = starknet_core::types::L1HandlerTransaction {
                transaction_hash,
                version: 0,
                nonce: tx.nonce,
                contract_address: tx.contract_address.into(),
                entry_point_selector: tx.entry_point_selector.into(),
                calldata: cast_vec_of_felt_252_wrappers(tx.calldata),
            };

            starknet_core::types::Transaction::L1Handler(tx)
        }
    }
}
