import {
  Account,
  InvokeFunctionResponse,
  RpcProvider,
  ec,
  number,
} from "starknet";
import BN__default from "bn.js";
import {
  ARGENT_CONTRACT_ADDRESS,
  FEE_TOKEN_ADDRESS,
  SIGNER_PRIVATE,
} from "../tests/constants";
import { numberToU8a } from "@polkadot/util";

export type BigNumberish = string | number | BN__default;

// Convert a BigNumberish to a hex string
export function toHex(value: BigNumberish) {
  return number.toHex(number.toBN(value));
}

// Convert a string or number to a BN
export function toBN(value: BigNumberish) {
  return number.toBN(value);
}

// Convert a BigNumberish to a 32 byte uint array
export function numberToU832Bytes(value: number) {
  return numberToU8a(value, 256);
}

export async function rpcTransfer(
  providerRPC: RpcProvider,
  nonce: { value: number },
  recipient: string,
  transferAmount: string,
  maxFee?: number
): Promise<InvokeFunctionResponse> {
  const keyPair = ec.getKeyPair(SIGNER_PRIVATE);
  const account = new Account(providerRPC, ARGENT_CONTRACT_ADDRESS, keyPair);

  const invokeResponse = account.execute(
    {
      contractAddress: FEE_TOKEN_ADDRESS,
      entrypoint: "transfer",
      calldata: [recipient, transferAmount, "0x0"],
    },
    undefined,
    {
      nonce: nonce.value,
      maxFee: maxFee ?? "123456",
    }
  );

  nonce.value++;

  return invokeResponse;
}
