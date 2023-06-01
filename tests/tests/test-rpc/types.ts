export interface InvokeTransaction {
  transaction_hash: string;
  calldata: string[];
  keys: string[];
  type: string;
  max_fee: string;
  version: string;
  signature: string[];
  nonce: string;
  sender_address: string;
}

export interface Block {
  status: string;
  transactions: string[];
}
