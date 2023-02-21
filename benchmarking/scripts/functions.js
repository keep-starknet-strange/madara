// SPDX-License-Identifier: Apache-2.0

const { Keyring } = require("@polkadot/keyring");

module.exports = { rpcMethods, runCairoProgram };

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

async function deployCairoProgram(userContext, events, done) {
  const { accountName } = userContext.vars;

  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri(`//${accountName}`);

  const extrisinc =
    userContext.api.tx.cairo.deployCairoAssemblyProgram(programId);
  // console.log(extrisinc)
  await extrisinc.signAndSend(alice, { nonce: -1 });

  return done();
}
