import { ec, number, Signature, addAddressPadding } from "starknet";

export function signTransaction(txHash: string, privateKey = ""): Signature {
  if (privateKey === "") {
    return [];
  }
  const starkKeyPair = ec.getKeyPair(privateKey);

  return ec
    .sign(starkKeyPair, txHash)
    .map((s) => addAddressPadding(number.toHexString(s)));
}
