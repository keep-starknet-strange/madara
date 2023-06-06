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

export interface TransactionReceipt {
  transaction_hash: string;
  actual_fee: string;
  status: string;
  block_hash?: string;
  block_number?: string;
  type: string;
  messages_sent: Array<MsgToL1>;
  events: Array<string>;
}

interface MsgToL1 {
  to_address: string;
  payload: Array<string>;
}

export interface Block {
  status: string;
  transactions: string[];
}
