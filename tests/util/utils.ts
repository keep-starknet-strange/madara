import {
  Account,
  InvokeFunctionResponse,
  RpcProvider,
  ec,
  number,
  stark,
} from "starknet";
import BN__default from "bn.js";
import { ARGENT_CONTRACT_ADDRESS, FEE_TOKEN_ADDRESS } from "../tests/constants";

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
  const priKey = stark.randomAddress();
  const keyPair = ec.getKeyPair(priKey);
  const account = new Account(providerRPC, ARGENT_CONTRACT_ADDRESS, keyPair);

  const resp = await account.execute(
    {
      contractAddress: ARGENT_CONTRACT_ADDRESS,
      entrypoint: "transfer",
      calldata: [
        FEE_TOKEN_ADDRESS, // CONTRACT ADDRESS
        "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e", // SELECTOR (transfer)
        "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA SIZE
        recipient,
        transferAmount,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
      ],
    },
    undefined,
    {
      nonce,
      maxFee: maxFee ?? "123456",
    }
  );

  return resp;
}
