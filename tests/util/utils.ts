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

export type BigNumberish = string | number | BN__default;

// Convert a BigNumberish to a hex string
export function toHex(value: BigNumberish) {
  return number.toHex(number.toBN(value));
}

export async function rpcTransfer(
  providerRPC: RpcProvider,
  nonce: number,
  recipient: string,
  transferAmount: string,
  maxFee?: number
): Promise<InvokeFunctionResponse> {
  const keyPair = ec.getKeyPair(SIGNER_PRIVATE);
  const account = new Account(providerRPC, ARGENT_CONTRACT_ADDRESS, keyPair);

  return account.execute(
    {
      contractAddress: FEE_TOKEN_ADDRESS,
      entrypoint: "transfer",
      calldata: [recipient, transferAmount, "0x0"],
    },
    undefined,
    {
      nonce,
      maxFee: maxFee ?? "123456",
    }
  );
}
