import {
  Account,
  BigNumberish,
  InvokeFunctionResponse,
  RpcProvider,
  hash,
  num,
  number,
} from "starknet";
import {
  ARGENT_CONTRACT_ADDRESS,
  FEE_TOKEN_ADDRESS,
  SIGNER_PRIVATE,
} from "../tests/constants";
import { numberToU8a } from "@polkadot/util";

// Convert a BigNumberish to a hex string
export function toHex(value: BigNumberish) {
  return num.toHex(value);
}

// Convert a BigNumberish to a 32 byte uint array
export function numberToU832Bytes(value: number) {
  return numberToU8a(value, 256);
}

// Calculate the StarkNet keccak hash of a string
export function starknetKeccak(value: string) {
  return hash.starknetKeccak(value);
}

// Clean a hex string, remove leading 0's
export function cleanHex(value: string) {
  const cleaned = number.cleanHex(value);
  if (cleaned === "0x") {
    return "0x0";
  }
  return cleaned;
}

export async function rpcTransfer(
  providerRPC: RpcProvider,
  nonce: { value: number },
  recipient: string,
  transferAmount: string,
  maxFee?: number,
): Promise<InvokeFunctionResponse> {
  const account = new Account(
    providerRPC,
    ARGENT_CONTRACT_ADDRESS,
    SIGNER_PRIVATE,
  );

  const invokeResponse = account.execute(
    {
      contractAddress: FEE_TOKEN_ADDRESS,
      entrypoint: "transfer",
      calldata: [recipient, transferAmount, "0x0"],
    },
    undefined,
    {
      nonce: nonce.value,
      maxFee: maxFee ?? "12345678",
    },
  );

  nonce.value++;

  return invokeResponse;
}
