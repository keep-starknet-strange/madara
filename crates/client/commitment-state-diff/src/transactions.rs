use bitvec::prelude::*;
use bonsai_trie::databases::HashMapDb;
use bonsai_trie::id::{BasicId, BasicIdBuilder};
use bonsai_trie::{BonsaiStorage, BonsaiStorageConfig};
use mc_db::storage_handler::bonsai_identifier;
use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use mp_hashers::HasherT;
use mp_transactions::compute_hash::ComputeTransactionHash;
use rayon::prelude::*;
use starknet_api::transaction::Transaction;
use starknet_ff::FieldElement;
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::Pedersen;

/// Compute the combined hash of the transaction hash and the signature.
///
/// Since the transaction hash doesn't take the signature values as its input
/// computing the transaction commitent uses a hash value that combines
/// the transaction hash with the array of signature values.
///
/// # Arguments
///
/// * `transaction` - The transaction to compute the hash of.
///
/// # Returns
///
/// The transaction hash with signature.
pub fn calculate_transaction_hash_with_signature<H: HasherT>(
    transaction: &Transaction,
    chain_id: Felt252Wrapper,
    block_number: u64,
) -> FieldElement
where
    H: HasherT,
{
    let include_signature = block_number >= 61394;

    let (signature_hash, tx_hash) = rayon::join(
        || match transaction {
            Transaction::Invoke(invoke_tx) => {
                // Include signatures for Invoke transactions or for all transactions
                let signature = invoke_tx.signature();

                H::compute_hash_on_elements(
                    &signature.0.iter().map(|x| Felt252Wrapper::from(*x).into()).collect::<Vec<FieldElement>>(),
                )
            }
            Transaction::Declare(declare_tx) => {
                // Include signatures for Declare transactions if the block number is greater than 61394 (mainnet)
                if include_signature {
                    let signature = declare_tx.signature();

                    H::compute_hash_on_elements(
                        &signature.0.iter().map(|x| Felt252Wrapper::from(*x).into()).collect::<Vec<FieldElement>>(),
                    )
                } else {
                    H::compute_hash_on_elements(&[])
                }
            }
            Transaction::DeployAccount(deploy_account_tx) => {
                // Include signatures for DeployAccount transactions if the block number is greater than 61394
                // (mainnet)
                if include_signature {
                    let signature = deploy_account_tx.signature();

                    H::compute_hash_on_elements(
                        &signature.0.iter().map(|x| Felt252Wrapper::from(*x).into()).collect::<Vec<FieldElement>>(),
                    )
                } else {
                    H::compute_hash_on_elements(&[])
                }
            }
            Transaction::L1Handler(_) => H::compute_hash_on_elements(&[]),
            _ => H::compute_hash_on_elements(&[]),
        },
        || Felt252Wrapper::from(transaction.compute_hash::<H>(chain_id, false, Some(block_number)).0).into(),
    );

    H::hash_elements(tx_hash, signature_hash)
}

/// Calculate the transaction commitment in memory using HashMapDb (which is more efficient for this
/// usecase).
///
/// # Arguments
///
/// * `transactions` - The transactions of the block
/// * `chain_id` - The current chain id
/// * `block_number` - The current block number
///
/// # Returns
///
/// The transaction commitment as `Felt252Wrapper`.
pub fn memory_transaction_commitment(
    transactions: &[Transaction],
    chain_id: Felt252Wrapper,
    block_number: u64,
) -> Result<Felt252Wrapper, String> {
    // TODO @cchudant refacto/optimise this function
    let config = BonsaiStorageConfig::default();
    let bonsai_db = HashMapDb::<BasicId>::default();
    let mut bonsai_storage =
        BonsaiStorage::<_, _, Pedersen>::new(bonsai_db, config).expect("Failed to create bonsai storage");
    let identifier = bonsai_identifier::TRANSACTION;

    // transaction hashes are computed in parallel
    let txs = transactions
        .par_iter()
        .map(|tx| calculate_transaction_hash_with_signature::<PedersenHasher>(tx, chain_id, block_number))
        .collect::<Vec<_>>();

    // once transaction hashes have finished computing, they are inserted into the local Bonsai db
    for (i, tx_hash) in txs.into_iter().enumerate() {
        let key = BitVec::from_vec(i.to_be_bytes().to_vec());
        let value = Felt::from(Felt252Wrapper::from(tx_hash));
        bonsai_storage.insert(identifier, key.as_bitslice(), &value).expect("Failed to insert into bonsai storage");
    }

    let mut id_builder = BasicIdBuilder::new();
    let id = id_builder.new_id();

    bonsai_storage.commit(id).expect("Failed to commit to bonsai storage");
    let root_hash = bonsai_storage.root_hash(identifier).expect("Failed to get root hash");

    Ok(Felt252Wrapper::from(root_hash))
}
