// SPDX-License-Identifier: Apache-2.0

const { Keyring } = require("@polkadot/keyring");
const fibJson = require("../../pallets/cairo/src/execution/samples/fib.json");
const addJson = require("../../pallets/cairo/src/execution/samples/add.json");

module.exports = {
  rpcMethods,
  runCairoProgram,
  executeCairoProgram,
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
