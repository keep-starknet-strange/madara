import fs from "fs";
import { CompiledContract, CompiledSierraCasm, json } from "starknet";

export const TEST_CONTRACT_ADDRESS =
  "0x0000000000000000000000000000000000000000000000000000000000001111";

export const ACCOUNT_CONTRACT =
  "0x0000000000000000000000000000000000000000000000000000000000000001";

export const CAIRO_1_ACCOUNT_CONTRACT =
  "0x0000000000000000000000000000000000000000000000000000000000000004";

export const CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH =
  "0x35ccefcf9d5656da623468e27e682271cd327af196785df99e7fee1436b6276";

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
export const DEPLOY_ACCOUNT_COST =
  "0x00000000000000000000000000000000000000000000000000000000ffffffff";
export const CONTRACT_ADDRESS =
  "0x0000000000000000000000000000000000000000000000000000000000000001";
export const FEE_TOKEN_ADDRESS =
  "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";
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

export const UDC_CONTRACT_ADDRESS =
  "0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf";
export const UDC_CLASS_HASH = "0x90000";

// Contract classes
export const ERC20_CONTRACT: CompiledContract = json.parse(
  fs.readFileSync("../cairo-contracts/build/ERC20.json").toString("ascii"),
);
export const ERC721_CONTRACT: CompiledContract = json.parse(
  fs.readFileSync("../cairo-contracts/build/ERC721.json").toString("ascii"),
);
export const TEST_CONTRACT: CompiledContract = json.parse(
  fs.readFileSync("../cairo-contracts/build/test.json").toString("ascii"),
);
export const TEST_CAIRO_1_SIERRA: CompiledContract = json.parse(
  fs
    .readFileSync("../cairo-contracts/build/cairo_1/HelloStarknet.sierra.json")
    .toString("ascii"),
);
export const ERC20_CAIRO_1_SIERRA: CompiledContract = json.parse(
  fs
    .readFileSync("../cairo-contracts/build/cairo_1/erc20.sierra.json")
    .toString("ascii"),
);
export const TEST_CAIRO_1_CASM: CompiledSierraCasm = json.parse(
  fs
    .readFileSync("../cairo-contracts/build/cairo_1/HelloStarknet.casm.json")
    .toString("ascii"),
);
export const ERC20_CAIRO_1_CASM: CompiledSierraCasm = json.parse(
  fs
    .readFileSync("../cairo-contracts/build/cairo_1/erc20.casm.json")
    .toString("ascii"),
);
