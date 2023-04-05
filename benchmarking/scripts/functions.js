// SPDX-License-Identifier: Apache-2.0

const { Keyring } = require("@polkadot/keyring");
const fibJson = require("../../pallets/cairo/src/execution/samples/fib.json");
const addJson = require("../../pallets/cairo/src/execution/samples/add.json");
const { transfer, deploy, declare, initialize } = require("../../tests/util/starknet.ts");

module.exports = {
  rpcMethods,
  runCairoProgram,
  executeCairoProgram,
  executeERC20Transfer,
};

function rpcMethods(userContext, events, done) {
  const data = {"id":1, "jsonrpc":"2.0", "method": "rpc_methods"};
  // set the "data" variable for the virtual user to use in the subsequent action
  userContext.vars.data = data;
  return done();
}

async function runCairoProgram(userContext, events, done) {
  const { programId, accountName } = userContext.vars;

   const keyring = new Keyring({ type: "sr25519" });
   const alice = keyring.addFromUri(`//${accountName}`);

  const extrisinc = userContext.api.tx.cairo.executeHardcodedCairoAssemblyProgram(
    programId
  );
  // console.log(extrisinc)
  await extrisinc.signAndSend(alice, {nonce: -1})

  return done();
}

async function executeCairoProgram(userContext, events, done) {
  const { accountName, programId, programs } = userContext.vars;

  // Deploy program if it doesn't exist
  if (!programs[programId]) {
    await _deployCairoProgram(userContext);
  }

  const keyring = new Keyring({ type: "sr25519" });
  const user = keyring.addFromUri(`//${accountName}`);

  // TODO: change this to a switch statement
  const cairoProgramId =
    programId === 0
      ? "0xd46a8b84ce2ec2be26482f551b619f5826d0d79cbb7b4685945c13badbb7383d"
      : "0x0";

  const extrisinc = userContext.api.tx.cairo.executeCairoAssemblyProgram(
    cairoProgramId,
    0
  );
  await extrisinc.signAndSend(user, { nonce: -1 });

  return done();
}

async function executeERC20Transfer(userContext, events, done) {
  const { accountName, deployed } = userContext.vars;

  const keyring = new Keyring({ type: "sr25519" });
  const user = keyring.addFromUri(`//${accountName}`);

  const contractAddress =
    "0x0000000000000000000000000000000000000000000000000000000000000101";
  const amount =
    "0x0000000000000000000000000000000000000000000000000000000000000001";
  const mintAmount =
    "0x0000000000000000000000000000000000000000000000000000000000001000";
  const tokenClassHash =
    "0x025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918";


  // Setup contract if it doesn't exist
  if (!deployed[tokenAddress]) {
    await declare(userContext.api, user, contractAddress, tokenClassHash);

    const tokenAddress = await deploy(userContext.api, user, contractAddress, tokenClassHash);

    await initialize(userContext.api, user, contractAddress, tokenAddress);

    await mint(userContext.api, user, contractAddress, tokenAddress, mintAmount);

    // Update userContext deployed dict
    userContext.vars.deployed = {...userContext.vars.deployed, [tokenAddress]: true};
  }


  await transfer(
    userContext.api,
    user,
    contractAddress,
    tokenAddress,
    contractAddress,
    amount
  );

  return done();
}

async function _deployCairoProgram(userContext) {
  const { accountName, programId } = userContext.vars;

  const keyring = new Keyring({ type: "sr25519" });
  const user = keyring.addFromUri(`//${accountName}`);

  const programJson = getJSONFromProgramId(programId);
  const bytes = Buffer.from(JSON.stringify(programJson));

  const extrisinc = userContext.api.tx.cairo.deployCairoAssemblyProgram(bytes.toString());
  await extrisinc.signAndSend(user, { nonce: -1 });

  // Update userContext programs dict
  userContext.vars.programs = {...userContext.vars.programs, [programId]: bytes};

  return;
}

// Load JSON files from ../../pallets/cairo/src/execution/samples
function getJSONFromProgramId(programId) {
  let programJson;
  switch (programId) {
    case 0:
      programJson = fibJson;
      break;

    case 1:
      programJson = addJson;
      break;

    default:
      throw Error("Invalid programId")
  }
  return programJson;
}
