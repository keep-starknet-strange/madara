const { ApiPromise, WsProvider } = require('@polkadot/api');
const util_crypto = require('@polkadot/util-crypto');

main().then(() => console.log('Done!'));

const CAIRO_MODULE = "Cairo";

async function main(){
    const wsProvider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider: wsProvider });
    console.log("Genesis hash: ", api.genesisHash.toHex());
}
