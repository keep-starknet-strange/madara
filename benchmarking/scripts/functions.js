// Copyright 2021-2022 Dwellir AB authors & contributors
// SPDX-License-Identifier: Apache-2.0

module.exports = { rpcMethods, getHash, someComplexCall };

function rpcMethods(userContext, events, done) {
  const data = {"id":1, "jsonrpc":"2.0", "method": "rpc_methods"};
  // set the "data" variable for the virtual user to use in the subsequent action
  userContext.vars.data = data;
  return done();
}

function getHash(userContext, events, done) {
  const { hash } = userContext.vars.data;

  userContext.vars.data = hash;
  return done();
}

async function someComplexCall(userContext, events, done) {
  const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
  const [{ nonce: accountNonce }, now] = await Promise.all([
   userContext.api.query.system.account(ALICE),
   userContext.api.query.timestamp.now()
  ]);

  userContext.vars.accountNonce = accountNonce;
  userContext.vars.now = now;
  return done();
}

async function runCairoProgram(userContext, events, done) {

  

  return done();
}
