// SPDX-License-Identifier: Apache-2.0

const { Keyring } = require("@polkadot/keyring");
const fibJson = require("../../ressources/fib.json");
const addJson = require("../../ressources/add.json");
const accountJson = require("../../ressources/account.json");
const erc20Json = require("../../ressources/erc20.json");
const { u8aToHex } = require("@polkadot/util");

const ENTRYPOINTS = {
  'CONSTRUCTOR': 0,
  'EXTERNAL': 1,
  'L1_HANDLER': 2,
}

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
    "0x0000000000000000000000000000000000000000000000000000000000000101";
  const accountClassHash =
    "0x025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
  const tokenClassHash =
    "0x0624EBFb99865079bd58CFCFB925B6F5Ce940D6F6e41E118b8A72B7163fB435c";

  // // Convert Uint8Array to hex string
  // const hexStringProgram = u8aToHex(
  //   new TextEncoder().encode(JSON.stringify(accountJson.program))
  // );

  // let encodedData = new Map(Object.entries(accountJson.entry_points_by_type).map(([k, v]) => [ENTRYPOINTS[k], v]));
  // encodedData = u8aToHex(new TextEncoder().encode(encodedData));

  // // Declare account class hash
  // const tx_declare = {
  //   version: 1, // version of the transaction
  //   hash: "", // leave empty for now, will be filled in by the runtime
  //   signature: [], // leave empty for now, will be filled in when signing the transaction
  //   events: [], // empty vector for now, will be filled in by the runtime
  //   sender_address: contractAddress, // address of the sender contract
  //   nonce: 0, // nonce of the transaction
  //   callEntrypoint: {
  //     // call entrypoint
  //     classHash: accountClassHash, // class hash of the contract
  //     entrypointSelector: null, // function selector of the transfer function
  //     calldata: [], // empty vector for now, will be filled in by the runtime
  //     storageAddress: contractAddress,
  //     callerAddress: contractAddress,
  //   },
  //   contractClass: {
  //     program: hexStringProgram,
  //     entryPointsByType: encodedData,
  //   },
  // };

  // const extrisinc_declare =
  //   userContext.api.tx.starknet.addDeclareTransaction(tx_declare);
  // const signedTxDeclare = await extrisinc_declare.signAsync(user, {
  //   nonce: -1,
  // });
  // const resultDeclare = await signedTxDeclare.send();

  // // Deploy account contract
  // let tx_deploy = {
  //   version: 1, // version of the transaction
  //   hash: "", // leave empty for now, will be filled in by the runtime
  //   signature: [], // leave empty for now, will be filled in when signing the transaction
  //   events: [], // empty vector for now, will be filled in by the runtime
  //   sender_address: contractAddress, // address of the sender contract
  //   nonce: 1, // nonce of the transaction
  //   callEntrypoint: {
  //     // call entrypoint
  //     classHash: accountClassHash, // class hash of the contract
  //     entrypointSelector: null, // function selector of the transfer function
  //     calldata: [], // empty vector for now, will be filled in by the runtime
  //     storageAddress: contractAddress,
  //     callerAddress: contractAddress,
  //   },
  //   contractClass: null,
  // };

  // const extrisinc_deploy =
  //   userContext.api.tx.starknet.addDeployAccountTransaction(tx_declare);
  // const signedTxDeploy = await extrisinc_deploy.signAsync(user, {
  //   nonce: -1,
  // });
  // const resultDeploy = await signedTxDeploy.send();

  // Execute transaction
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
        "0x0000000000000000000000000000000000000000000000000000000000001001", // contract address
        "0x00e7def693d16806ca2a2f398d8de5951344663ba77f340ed7a958da731872fc", // selector
        "0x0000000000000000000000000000000000000000000000000000000000000001", // calldata length
        "0x0000000000000000000000000000000000000000000000000000000000000019", // calldata
      ],
      storageAddress: contractAddress,
      callerAddress: contractAddress,
    },
    contractClass: null,
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
