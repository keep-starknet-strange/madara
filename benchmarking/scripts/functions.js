// SPDX-License-Identifier: Apache-2.0
const { ACCOUNT_CONTRACT, ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE, ERC20_CONTRACT_ADDRESS} = require("../../tests/build/tests/tests/constants");
const {
  initialize,
  mint,
  declare,
  deploy,
  transfer,
  mintERC721,
} = require("../../tests/build/tests/util/starknet");

const { numberToHex } = require("@polkadot/util");
const { Account, RpcProvider, ec, hash, number, constants } = require("starknet");

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
  // const { target } = userContext.vars;
  // console.log("RPC :", target.replace("ws", "http"));

  // let providerRPC = new RpcProvider({
  //   nodeUrl: target.replace("ws", "http"),
  //   retries: 3,
  // }); // substrate node

  const { nonce } = userContext.vars;
  const amount =
    "0x0000000000000000000000000000000000000000000000000000000000000001";

  // TODO: Once declare bug fixed we can call _setupToken and remove hardcoded address
    
  const calldata = [
    ERC20_CONTRACT_ADDRESS, // CONTRACT ADDRESS
    "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e", // SELECTOR (transfer)
    "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA SIZE
    ACCOUNT_CONTRACT,
    amount,
    "0x0000000000000000000000000000000000000000000000000000000000000000",
  ];

  const txHash = hash.calculateTransactionHashCommon(
    constants.TransactionHashPrefix.INVOKE,
    1,
    ARGENT_CONTRACT_ADDRESS,
    0,
    calldata,
    0,
    constants.StarknetChainId.TESTNET,
    [nonce]
  );

  const keyPair = ec.getKeyPair(SIGNER_PRIVATE);
  const signature = ec.sign(keyPair, txHash);
  console.log("KEY PAIR :", keyPair);
  // const account = new Account(
  //   providerRPC,
  //   ARGENT_CONTRACT_ADDRESS,
  //   keyPair
  // );
  transfer(
    userContext.api,
    ARGENT_CONTRACT_ADDRESS,
    ERC20_CONTRACT_ADDRESS,
    ACCOUNT_CONTRACT,
    amount,
    nonce,
    [
      "0x" + number.toHexString(signature[0]).slice(2).padStart(64, "0"), 
      "0x" + number.toHexString(signature[1]).slice(2).padStart(64, "0")
    ]
  ).send();
  // ).signAndSend(keyPair);

  // Update userContext nonce
  userContext.vars.nonce = nonce + 1;

  return done();
}

async function executeERC721Mint(userContext, events, done) {
  const { nonce } = userContext.vars;

  mintERC721(
    userContext.api,
    ARGENT_CONTRACT_ADDRESS,
    CONTRACT_ADDRESS,
    numberToHex(nonce, 256),
    nonce
  ).send();

  // Update userContext nonce
  userContext.vars.nonce = nonce + 1;

  return done();
}
