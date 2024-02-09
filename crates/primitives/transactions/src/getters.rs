use alloc::vec::Vec;

use mp_felt::Felt252Wrapper;

use super::{DeclareTransaction, DeployAccountTransaction, InvokeTransaction, Transaction, UserTransaction};
use crate::{
    DeclareTransactionV0, DeclareTransactionV1, DeclareTransactionV2, HandleL1MessageTransaction, InvokeTransactionV0,
    InvokeTransactionV1, UserOrL1HandlerTransaction,
};

impl Transaction {
    pub fn signature(&self) -> Vec<Felt252Wrapper> {
        match self {
            Transaction::Declare(tx) => tx.signature().clone(),
            Transaction::DeployAccount(tx) => tx.signature().clone(),
            Transaction::Invoke(tx) => tx.signature().clone(),
            Transaction::L1Handler(_) => Vec::new(),
        }
    }
}

impl UserTransaction {
    pub fn sender_address(&self) -> Felt252Wrapper {
        match self {
            UserTransaction::Declare(tx, _) => *tx.sender_address(),
            UserTransaction::DeployAccount(tx) => tx.account_address(),
            UserTransaction::Invoke(tx) => *tx.sender_address(),
        }
    }

    pub fn signature(&self) -> &Vec<Felt252Wrapper> {
        match self {
            UserTransaction::Declare(tx, _) => tx.signature(),
            UserTransaction::DeployAccount(tx) => tx.signature(),
            UserTransaction::Invoke(tx) => tx.signature(),
        }
    }

    pub fn max_fee(&self) -> &u128 {
        match self {
            UserTransaction::Declare(tx, _) => tx.max_fee(),
            UserTransaction::DeployAccount(tx) => tx.max_fee(),
            UserTransaction::Invoke(tx) => tx.max_fee(),
        }
    }

    pub fn calldata(&self) -> Option<&Vec<Felt252Wrapper>> {
        match self {
            UserTransaction::Declare(..) => None,
            UserTransaction::DeployAccount(tx) => Some(tx.calldata()),
            UserTransaction::Invoke(tx) => Some(tx.calldata()),
        }
    }

    pub fn nonce(&self) -> Option<&Felt252Wrapper> {
        match self {
            UserTransaction::Declare(tx, _) => Some(tx.nonce()),
            UserTransaction::DeployAccount(tx) => Some(tx.nonce()),
            UserTransaction::Invoke(tx) => tx.nonce(),
        }
    }

    pub fn offset_version(&self) -> bool {
        match self {
            UserTransaction::Declare(tx, _) => tx.offset_version(),
            UserTransaction::DeployAccount(tx) => tx.offset_version(),
            UserTransaction::Invoke(tx) => tx.offset_version(),
        }
    }
}

impl DeclareTransaction {
    pub fn sender_address(&self) -> &Felt252Wrapper {
        match self {
            DeclareTransaction::V0(tx) => &tx.sender_address,
            DeclareTransaction::V1(tx) => &tx.sender_address,
            DeclareTransaction::V2(tx) => &tx.sender_address,
        }
    }

    pub fn signature(&self) -> &Vec<Felt252Wrapper> {
        match self {
            DeclareTransaction::V0(tx) => &tx.signature,
            DeclareTransaction::V1(tx) => &tx.signature,
            DeclareTransaction::V2(tx) => &tx.signature,
        }
    }

    pub fn max_fee(&self) -> &u128 {
        match self {
            DeclareTransaction::V0(tx) => &tx.max_fee,
            DeclareTransaction::V1(tx) => &tx.max_fee,
            DeclareTransaction::V2(tx) => &tx.max_fee,
        }
    }

    pub fn nonce(&self) -> &Felt252Wrapper {
        match self {
            DeclareTransaction::V0(tx) => &tx.nonce,
            DeclareTransaction::V1(tx) => &tx.nonce,
            DeclareTransaction::V2(tx) => &tx.nonce,
        }
    }

    pub fn class_hash(&self) -> &Felt252Wrapper {
        match self {
            DeclareTransaction::V0(tx) => &tx.class_hash,
            DeclareTransaction::V1(tx) => &tx.class_hash,
            DeclareTransaction::V2(tx) => &tx.class_hash,
        }
    }

    pub fn compiled_class_hash(&self) -> Option<&Felt252Wrapper> {
        match self {
            DeclareTransaction::V0(_) => None,
            DeclareTransaction::V1(_) => None,
            DeclareTransaction::V2(tx) => Some(&tx.compiled_class_hash),
        }
    }

    pub fn offset_version(&self) -> bool {
        match self {
            // we don't accept V0 txs from the RPC
            DeclareTransaction::V0(_) => false,
            DeclareTransaction::V1(tx) => tx.offset_version,
            DeclareTransaction::V2(tx) => tx.offset_version,
        }
    }
}

impl DeployAccountTransaction {
    pub fn signature(&self) -> &Vec<Felt252Wrapper> {
        &self.signature
    }

    pub fn max_fee(&self) -> &u128 {
        &self.max_fee
    }

    pub fn calldata(&self) -> &Vec<Felt252Wrapper> {
        &self.constructor_calldata
    }

    pub fn nonce(&self) -> &Felt252Wrapper {
        &self.nonce
    }

    pub fn account_address(&self) -> Felt252Wrapper {
        Felt252Wrapper(self.get_account_address())
    }

    pub fn class_hash(&self) -> &Felt252Wrapper {
        &self.class_hash
    }

    pub fn offset_version(&self) -> bool {
        self.offset_version
    }
}

impl InvokeTransaction {
    pub fn sender_address(&self) -> &Felt252Wrapper {
        match self {
            InvokeTransaction::V0(tx) => &tx.contract_address,
            InvokeTransaction::V1(tx) => &tx.sender_address,
        }
    }

    pub fn signature(&self) -> &Vec<Felt252Wrapper> {
        match self {
            InvokeTransaction::V0(tx) => &tx.signature,
            InvokeTransaction::V1(tx) => &tx.signature,
        }
    }

    pub fn max_fee(&self) -> &u128 {
        match self {
            InvokeTransaction::V0(tx) => &tx.max_fee,
            InvokeTransaction::V1(tx) => &tx.max_fee,
        }
    }

    pub fn calldata(&self) -> &Vec<Felt252Wrapper> {
        match self {
            InvokeTransaction::V0(tx) => &tx.calldata,
            InvokeTransaction::V1(tx) => &tx.calldata,
        }
    }

    pub fn nonce(&self) -> Option<&Felt252Wrapper> {
        match self {
            InvokeTransaction::V0(_) => None,
            InvokeTransaction::V1(tx) => Some(&tx.nonce),
        }
    }

    pub fn offset_version(&self) -> bool {
        match self {
            // we don't accept V0 txs from the RPC
            InvokeTransaction::V0(_) => false,
            InvokeTransaction::V1(tx) => tx.offset_version,
        }
    }
}

pub trait TransactionVersion {
    fn version(&self) -> u8;
}

impl TransactionVersion for UserTransaction {
    #[inline(always)]
    fn version(&self) -> u8 {
        match self {
            UserTransaction::Declare(tx, _) => tx.version(),
            UserTransaction::DeployAccount(tx) => tx.version(),
            UserTransaction::Invoke(tx) => tx.version(),
        }
    }
}

impl TransactionVersion for Transaction {
    #[inline(always)]
    fn version(&self) -> u8 {
        match self {
            Transaction::Declare(tx) => tx.version(),
            Transaction::DeployAccount(tx) => tx.version(),
            Transaction::Invoke(tx) => tx.version(),
            Transaction::L1Handler(tx) => tx.version(),
        }
    }
}

impl TransactionVersion for UserOrL1HandlerTransaction {
    #[inline(always)]
    fn version(&self) -> u8 {
        match self {
            UserOrL1HandlerTransaction::User(tx) => tx.version(),
            UserOrL1HandlerTransaction::L1Handler(tx, _) => tx.version(),
        }
    }
}

impl TransactionVersion for InvokeTransaction {
    #[inline(always)]
    fn version(&self) -> u8 {
        match self {
            InvokeTransaction::V0(tx) => tx.version(),
            InvokeTransaction::V1(tx) => tx.version(),
        }
    }
}

impl TransactionVersion for InvokeTransactionV0 {
    #[inline(always)]
    fn version(&self) -> u8 {
        0
    }
}

impl TransactionVersion for InvokeTransactionV1 {
    #[inline(always)]
    fn version(&self) -> u8 {
        1
    }
}

impl TransactionVersion for DeclareTransaction {
    #[inline(always)]
    fn version(&self) -> u8 {
        match self {
            DeclareTransaction::V0(tx) => tx.version(),
            DeclareTransaction::V1(tx) => tx.version(),
            DeclareTransaction::V2(tx) => tx.version(),
        }
    }
}

impl TransactionVersion for DeclareTransactionV0 {
    #[inline(always)]
    fn version(&self) -> u8 {
        0
    }
}

impl TransactionVersion for DeclareTransactionV1 {
    #[inline(always)]
    fn version(&self) -> u8 {
        1
    }
}

impl TransactionVersion for DeclareTransactionV2 {
    #[inline(always)]
    fn version(&self) -> u8 {
        2
    }
}

impl TransactionVersion for DeployAccountTransaction {
    #[inline(always)]
    fn version(&self) -> u8 {
        1
    }
}

impl TransactionVersion for HandleL1MessageTransaction {
    #[inline(always)]
    fn version(&self) -> u8 {
        0
    }
}
