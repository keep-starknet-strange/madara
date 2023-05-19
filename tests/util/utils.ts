import { number } from "starknet";
import BN__default from "bn.js";

export type BigNumberish = string | number | BN__default;

// Convert a BigNumberish to a hex string
export function toHex(value: BigNumberish) {
  return number.toHex(number.toBN(value));
}
