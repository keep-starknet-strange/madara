// SPDX-License-Identifier: Apache-2.0

const { Keyring } = require("@polkadot/keyring");
const {
  batchTransfer,
  initialize,
  mint,
  declare,
  deploy,
  transfer,
} = require("../../tests/build/util/starknet");

module.exports = {
  rpcMethods,
  executeERC20Transfer,
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

  await transfer(
    userContext.api,
    contractAddress,
    "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00",
    contractAddress,
    amount,
    nonce
  ).send();

  // Update userContext nonce
  userContext.vars.nonce = nonce + 1;

  return done();
}

async function _setupToken(userContext, user, contractAddress) {
  const { deployed } = userContext.vars;

  const mintAmount =
    "0x0000000000000000000000000000000000000000000000000000000000001000";
  const tokenClassHash =
    "0x025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918";

  // Setup token contract if it doesn't exist
  let tokenAddress;
  if (!deployed[tokenClassHash]) {
    try {
      await declare(userContext.api, user, contractAddress, tokenClassHash);

      tokenAddress = await deploy(
        userContext.api,
        user,
        contractAddress,
        tokenClassHash
      );

      console.log("Deployed token address: ", tokenAddress);

      await initialize(userContext.api, user, contractAddress, tokenAddress);

      await mint(
        userContext.api,
        user,
        contractAddress,
        tokenAddress,
        mintAmount
      );

      // Update userContext deployed dict
      userContext.vars.deployed = {
        ...userContext.vars.deployed,
        [tokenClassHash]: true,
      };
    } catch (error) {
      console.error(error);
    }
  }

  return tokenAddress;
}
