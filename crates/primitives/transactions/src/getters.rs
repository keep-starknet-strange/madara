use alloc::vec::Vec;

use mp_felt::Felt252Wrapper;

use super::{DeclareTransaction, DeployAccountTransaction, InvokeTransaction, Transaction, UserTransaction};

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

    pub fn version(&self) -> u8 {
        match self {
            UserTransaction::Declare(tx, _) => tx.version(),
            UserTransaction::DeployAccount(tx) => tx.version(),
            UserTransaction::Invoke(tx) => tx.version(),
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

    pub fn version(&self) -> u8 {
        match self {
            DeclareTransaction::V0(_) => 0,
            DeclareTransaction::V1(_) => 1,
            DeclareTransaction::V2(_) => 2,
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

    pub fn version(&self) -> u8 {
        1
    }

    pub fn account_address(&self) -> Felt252Wrapper {
        Felt252Wrapper(self.get_account_address())
    }

    pub fn class_hash(&self) -> &Felt252Wrapper {
        &self.class_hash
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

    pub fn version(&self) -> u8 {
        match self {
            InvokeTransaction::V0(_) => 0,
            InvokeTransaction::V1(_) => 1,
        }
    }
}
