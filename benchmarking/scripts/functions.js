// SPDX-License-Identifier: Apache-2.0

const { Keyring } = require("@polkadot/keyring");
const fibJson = require("../../ressources/fib.json");
const addJson = require("../../ressources/add.json");

module.exports = {
  rpcMethods,
  runCairoProgram,
  executeCairoProgram,
  executeERC20Transfer,
};

function rpcMethods(userContext, events, done) {
  const data = { id: 1, jsonrpc: "2.0", method: "rpc_methods" };
  // set the "data" variable for the virtual user to use in the subsequent action
  userContext.vars.data = data;
  return done();
}

async function runCairoProgram(userContext, events, done) {
  const { programId, accountName } = userContext.vars;

  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri(`//${accountName}`);

  const extrisinc =
    userContext.api.tx.cairo.executeHardcodedCairoAssemblyProgram(programId);
  // console.log(extrisinc)
  await extrisinc.signAndSend(alice, { nonce: -1 });

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
  const { accountName } = userContext.vars;

  const keyring = new Keyring({ type: "sr25519" });
  const user = keyring.addFromUri(`//${accountName}`);

  const contractAddress =
    "0x02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
  const accountClassHash =
    "0x025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
    "0x0624EBFb99865079bd58CFCFB925B6F5Ce940D6F6e41E118b8A72B7163fB435c";

  const tx = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    events: [], // empty vector for now, will be filled in by the runtime
    sender_address: contractAddress, // address of the sender contract
    nonce: 0, // nonce of the transaction
    callEntrypoint: {
      // call entrypoint
      classHash: accountClassHash, // class hash of the contract
      entrypointSelector: null, // function selector of the transfer function
      calldata: [
        "0x0624ebfb99865079bd58cfcfb925b6f5ce940d6f6e41e118b8a72b7163fb435c",
        "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e",
        "0x0000000000000000000000000000000000000000000000000000000000000003",
        "0x0624EBFb99865079bd58CFCFB925B6F5Ce940D6F6e41E118b8A72B7163fB435c",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
      ],
    },
    storageAddress: contractAddress,
    callerAddress: contractAddress,
  };

  const extrisinc = userContext.api.tx.starknet.addInvokeTransaction(tx);
  const signedTx = await extrisinc.signAsync(user, { nonce: -1 });
  const result = await signedTx.send();

  return done();
}

async function _deployCairoProgram(userContext) {
  const { accountName, programId } = userContext.vars;

  const keyring = new Keyring({ type: "sr25519" });
  const user = keyring.addFromUri(`//${accountName}`);

  const programJson = getJSONFromProgramId(programId);
  const bytes = Buffer.from(JSON.stringify(programJson));

  const extrisinc = userContext.api.tx.cairo.deployCairoAssemblyProgram(
    bytes.toString()
  );
  await extrisinc.signAndSend(user, { nonce: -1 });

  // Update userContext programs dict
  userContext.vars.programs = {
    ...userContext.vars.programs,
    [programId]: bytes,
  };

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
      throw Error("Invalid programId");
  }
  return programJson;
}
