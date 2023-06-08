import fs from "fs";
import { CompiledContract, json } from "starknet";

export const TEST_CONTRACT_ADDRESS =
  "0x0000000000000000000000000000000000000000000000000000000000001111";

export const ACCOUNT_CONTRACT =
  "0x0000000000000000000000000000000000000000000000000000000000000001";

// https://github.com/keep-starknet-strange/madara/blob/main/crates/node/src/chain_spec.rs#L185-L186
export const ACCOUNT_CONTRACT_CLASS_HASH =
  "0x0279d77db761fba82e0054125a6fdb5f6baa6286fa3fb73450cc44d193c2d37f";

export const ARGENT_PROXY_CLASS_HASH =
  "0x0424b7f61e3c5dfd74400d96fdea7e1f0bf2757f31df04387eaa957f095dd7b9";
export const ARGENT_ACCOUNT_CLASS_HASH =
  "0x06f0d6f6ae72e1a507ff4b65181291642889742dbf8f1a53e9ec1c595d01ba7d";
export const SIGNER_PUBLIC =
  "0x03603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2";
export const SIGNER_PRIVATE =
  "0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d";
export const SALT =
  "0x0000000000000000000000000000000000000000000000000000000000001111";

// https://github.com/keep-starknet-strange/madara/blob/main/crates/node/src/chain_spec.rs#L191-L192
export const TEST_CONTRACT_CLASS_HASH =
  "0x0000000000000000000000000000000000000000000000000000000000001000";
export const MINT_AMOUNT =
  "0x0000000000000000000000000000000000000000000000000000000000000001";
export const CONTRACT_ADDRESS =
  "0x0000000000000000000000000000000000000000000000000000000000000001";
export const FEE_TOKEN_ADDRESS =
  "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d01";
export const TOKEN_CLASS_HASH =
  "0x0000000000000000000000000000000000000000000000000000000000010000";
export const ARGENT_CONTRACT_ADDRESS =
  "0x0000000000000000000000000000000000000000000000000000000000000002";

// Sequencer address
export const SEQUENCER_ADDRESS =
  "0x000000000000000000000000000000000000000000000000000000000000dead";

// Starknet testnet SN_GOERLI
export const CHAIN_ID_STARKNET_TESTNET = "0x534e5f474f45524c49";

export const NFT_CONTRACT_ADDRESS =
  "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d02";
export const NFT_CLASS_HASH = "0x90000";

// Contract classes

// For some reasons, the starknet-compile-deprecated command
// writes all the offsets as integer while the RPC returns them as hex string
// This chatGPT utils recursively updates them
function convertOffsetToHex(obj) {
  if (Array.isArray(obj)) {
    for (let i = 0; i < obj.length; i++) {
      obj[i] = convertOffsetToHex(obj[i]);
    }
  } else if (typeof obj === "object" && obj !== null) {
    for (const key in obj) {
      if (Object.prototype.hasOwnProperty.call(obj, key)) {
        obj[key] = convertOffsetToHex(obj[key]);
      }
    }
  } else if (typeof obj === "number" && Number.isInteger(obj) && obj >= 0) {
    obj = `0x${obj.toString(16)}`;
  }
  return obj;
}

const erc20Json = json.parse(
  fs.readFileSync("../cairo-contracts/build/ERC20.json").toString("ascii")
);
const testJson = json.parse(
  fs.readFileSync("../cairo-contracts/build/test.json").toString("ascii")
);
export const ERC20_CONTRACT: CompiledContract = {
  ...erc20Json,
  entry_points_by_type: convertOffsetToHex(erc20Json.entry_points_by_type),
};
export const TEST_CONTRACT: CompiledContract = {
  ...testJson,
  entry_points_by_type: convertOffsetToHex(testJson.entry_points_by_type),
};
