// SPDX-License-Identifier: Apache-2.0
const { TEST_CONTRACT_ADDRESS, ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE, ERC20_CONTRACT_ADDRESS, NFT_CONTRACT_ADDRESS, CHAIN_ID_STARKNET_TESTNET} = require("../../tests/build/tests/tests/constants");
const {
  initialize,
  mint,
  declare,
  deploy,
  transfer,
  mintERC721,
  calculateHexSignature
} = require("../../tests/build/tests/util/starknet");

const { numberToHex } = require("@polkadot/util");

module.exports = {
  rpcMethods,
  presignTransferTransactions,
  presignMintTransactions,
  executeERC20Transfer,
  executeERC721Mint,
};

function rpcMethods(userContext, events, done) {
  const data = { id: 1, jsonrpc: "2.0", method: "rpc_methods" };
  // set the "data" variable for the virtual user to use in the subsequent action
  userContext.vars.data = data;
  return done();
}

function presignTransferTransactions(userContext, events, done) {
  const amount =
    "0x0000000000000000000000000000000000000000000000000000000000000001";
  const calldata = [
    "0x0000000000000000000000000000000000000000000000000000000000000001", // CALL ARRAY LEN
    ERC20_CONTRACT_ADDRESS,                                               // TO
    "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e", // SELECTOR (transfer)
    "0x0000000000000000000000000000000000000000000000000000000000000000", // DATA OFFSET
    "0x0000000000000000000000000000000000000000000000000000000000000003", // DATA LEN
    "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA LEN
    TEST_CONTRACT_ADDRESS,
    amount,
    "0x0000000000000000000000000000000000000000000000000000000000000000"
  ];

  const signature = [];
  // i acts as nonce, starts from 0 and goes to the number of iterations for the benchmark
  for(let i = 0; i < 10000; i++) {
    signature[i] = calculateHexSignature(ARGENT_CONTRACT_ADDRESS, calldata, i, SIGNER_PRIVATE);
  }

  userContext.vars.signature = signature;
  return done();
}

function presignMintTransactions(userContext, events, done) {
  const calldata = [
    "0x0000000000000000000000000000000000000000000000000000000000000001", // CALL ARRAY LEN
    NFT_CONTRACT_ADDRESS,                                                 // TO
    "0x02f0b3c5710379609eb5495f1ecd348cb28167711b73609fe565a72734550354", // SELECTOR (mint)
    "0x0000000000000000000000000000000000000000000000000000000000000000", // DATA OFFSET
    "0x0000000000000000000000000000000000000000000000000000000000000003", // DATA LEN
    "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA LEN
    TEST_CONTRACT_ADDRESS,
    0,
    "0x0000000000000000000000000000000000000000000000000000000000000000"
  ];

  const signature = [];
  // i acts as nonce, starts from 0 and goes to the number of iterations for the benchmark
  for(let i = 0; i < 10000; i++) {
    // tokenID
    calldata[7] = numberToHex(i, 256);
    signature[i] = calculateHexSignature(ARGENT_CONTRACT_ADDRESS, calldata, i, SIGNER_PRIVATE);
  }

  userContext.vars.signature = signature;
  return done();
}

async function executeERC20Transfer(userContext, events, done) {
  const { signature } = userContext.vars;
  const { nonce } = userContext.vars;
  const amount =
    "0x0000000000000000000000000000000000000000000000000000000000000001";

  // TODO: Once declare bug fixed we can call _setupToken and remove hardcoded address

  transfer(
    userContext.api,
    ARGENT_CONTRACT_ADDRESS,
    ERC20_CONTRACT_ADDRESS,
    TEST_CONTRACT_ADDRESS,
    amount,
    nonce,
    signature[nonce]
  ).send();

  // Update userContext nonce
  userContext.vars.nonce = nonce + 1;

  return done();
}

async function executeERC721Mint(userContext, events, done) {
  const { signature } = userContext.vars;
  const { nonce } = userContext.vars;
  const tokenID = numberToHex(nonce, 256);

  mintERC721(
    userContext.api,
    ARGENT_CONTRACT_ADDRESS,
    TEST_CONTRACT_ADDRESS,
    tokenID,
    nonce,
    signature[nonce]
  ).send();

  // Update userContext nonce
  userContext.vars.nonce = nonce + 1;

  return done();
}