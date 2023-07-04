// SPDX-License-Identifier: Apache-2.0
const { ACCOUNT_CONTRACT, ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE, ERC20_CONTRACT_ADDRESS, NFT_CONTRACT_ADDRESS, CHAIN_ID_STARKNET_TESTNET} = require("../../tests/build/tests/tests/constants");
const {
  initialize,
  mint,
  declare,
  deploy,
  transfer,
  mintERC721,
} = require("../../tests/build/tests/util/starknet");

const { numberToHex } = require("@polkadot/util");
const { Signer, ec, hash, num, constants } = require("starknet");

module.exports = {
  rpcMethods,
  executeERC20Transfer,
  executeERC721Mint,
};

function rpcMethods(userContext, events, done) {
  const data = { id: 1, jsonrpc: "2.0", method: "rpc_methods" };
  // set the "data" variable for the virtual user to use in the subsequent action
  userContext.vars.data = data;
  return done();
}

async function executeERC20Transfer(userContext, events, done) {
  const { nonce } = userContext.vars;
  const amount =
    "0x0000000000000000000000000000000000000000000000000000000000000001";

  // TODO: Once declare bug fixed we can call _setupToken and remove hardcoded address
    
  const calldata = [
    "0x0000000000000000000000000000000000000000000000000000000000000001", // CALL ARRAY LEN
    ERC20_CONTRACT_ADDRESS,                                               // TO
    "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e", // SELECTOR (transfer)
    "0x0000000000000000000000000000000000000000000000000000000000000000", // DATA OFFSET
    "0x0000000000000000000000000000000000000000000000000000000000000003", // DATA LEN
    "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA LEN
    ACCOUNT_CONTRACT,
    amount,
    "0x0000000000000000000000000000000000000000000000000000000000000000"
  ];

  transfer(
    userContext.api,
    ARGENT_CONTRACT_ADDRESS,
    ERC20_CONTRACT_ADDRESS,
    ACCOUNT_CONTRACT,
    amount,
    nonce,
    calculateHexSignature(calldata, nonce)
  ).send();

  // Update userContext nonce
  userContext.vars.nonce = nonce + 1;

  return done();
}

async function executeERC721Mint(userContext, events, done) {
  const { nonce } = userContext.vars;
  const tokenID = numberToHex(nonce, 256);
    
  const calldata = [
    "0x0000000000000000000000000000000000000000000000000000000000000001", // CALL ARRAY LEN
    NFT_CONTRACT_ADDRESS,                                                 // TO
    "0x02f0b3c5710379609eb5495f1ecd348cb28167711b73609fe565a72734550354", // SELECTOR (mint)
    "0x0000000000000000000000000000000000000000000000000000000000000000", // DATA OFFSET
    "0x0000000000000000000000000000000000000000000000000000000000000003", // DATA LEN
    "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA LEN
    ACCOUNT_CONTRACT,
    tokenID,
    "0x0000000000000000000000000000000000000000000000000000000000000000"
  ];

  mintERC721(
    userContext.api,
    ARGENT_CONTRACT_ADDRESS,
    ACCOUNT_CONTRACT,
    tokenID,
    nonce,
    calculateHexSignature(calldata, nonce)
  ).send();

  // Update userContext nonce
  userContext.vars.nonce = nonce + 1;

  return done();
}

function calculateHexSignature(calldata, nonce) {
  const txHash = hash.calculateTransactionHashCommon(
    constants.TransactionHashPrefix.INVOKE,
    1,
    ARGENT_CONTRACT_ADDRESS,
    0,
    calldata,
    0,
    CHAIN_ID_STARKNET_TESTNET,
    [nonce]
  );

  const signature = ec.starkCurve.sign(txHash, SIGNER_PRIVATE);

  return [
    "0x" + num.toHexString(signature.r).slice(2).padStart(64, "0"), 
    "0x" + num.toHexString(signature.s).slice(2).padStart(64, "0")
  ];
}