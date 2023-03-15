const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require("@polkadot/keyring");
const util_crypto = require("@polkadot/util-crypto");

main().catch((error) => {
  console.error(error);
  process.exit(-1);
});

async function main() {
  const wsProvider = new WsProvider("ws://127.0.0.1:9944");
  const api = await ApiPromise.create({ provider: wsProvider });

  await getChainInfo(api);

  // Construct the keyring after the API (crypto has an async init)
  const keyring = new Keyring({ type: "sr25519" });

  // Add Alice to our keyring with a hard-derivation path (empty phrase, so uses dev)
  const alice = keyring.addFromUri("//Alice");
  const starknet_ping = api.tx.starknet.ping();

  // Sign and send the transaction using our account
  const hash = await starknet_ping.signAndSend(alice);
  console.log("starknet.ping extrinsic hash: ", hash.toHex());
}

async function getChainInfo(api) {
  // Retrieve the chain & node information information via rpc calls
  const [chain, nodeName, nodeVersion, genesisHash] = await Promise.all([
    api.rpc.system.chain(),
    api.rpc.system.name(),
    api.rpc.system.version(),
    api.genesisHash.toHex(),
  ]);
  console.log(
    `Connected to chain ${chain} using ${nodeName} v${nodeVersion} with genesis hash ${genesisHash}`
  );
}
