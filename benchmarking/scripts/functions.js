// SPDX-License-Identifier: Apache-2.0

const { ACCOUNT_CONTRACT } = require("../../tests/build/tests/tests/constants");
const {
  initialize,
  mint,
  declare,
  deploy,
  transfer,
  mintERC721,
} = require("../../tests/build/tests/util/starknet");

const { numberToHex } = require("@polkadot/util");

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
  const contractAddress =
    "0x0000000000000000000000000000000000000000000000000000000000000001";
  const amount =
    "0x0000000000000000000000000000000000000000000000000000000000000001";

  // TODO: Once declare bug fixed we can call _setupToken and remove hardcoded address

  transfer(
    userContext.api,
    contractAddress,
    "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00",
    "0x0000000000000000000000000000000000000000000000000000000000000002",
    amount,
    nonce
  ).send();

  // Update userContext nonce
  userContext.vars.nonce = nonce + 1;

  return done();
}

async function executeERC721Mint(userContext, events, done) {
  const { nonce } = userContext.vars;

  mintERC721(
    userContext.api,
    ACCOUNT_CONTRACT,
    "0x0000000000000000000000000000000000000000000000000000000000000002",
    numberToHex(nonce, 256),
    nonce
  ).send();

  // Update userContext nonce
  userContext.vars.nonce = nonce + 1;

  return done();
}
