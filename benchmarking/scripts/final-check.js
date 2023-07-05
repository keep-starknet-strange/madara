// Required imports
const { ApiPromise, WsProvider } = require("@polkadot/api");

async function main() {
  // Initialise the provider to connect to the local node
  const provider = new WsProvider("ws://127.0.0.1:9944");
  const api = await ApiPromise.create({ provider });

  const blockHash = await api.rpc.chain.getBlock();
  const blockNumber = blockHash.block.header.number;

  // check last 10 blocks for failed extrinsics
  for (let i = blockNumber.toNumber() - 9; i <= blockNumber.toNumber(); i++) {
    const hash = await api.rpc.chain.getBlockHash(i);
    const signedBlock = await api.rpc.chain.getBlock(hash);

    // get the api and events at a specific block
    const apiAt = await api.at(signedBlock.block.header.hash);
    const allRecords = await apiAt.query.system.events();

    signedBlock.block.extrinsics.forEach(
      ({ method: { method, section } }, index) => {
        allRecords
          .filter(
            ({ phase }) =>
              phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index),
          )
          .forEach(({ event }) => {
            // check for failed extrinsics
            if (api.events.system.ExtrinsicFailed.is(event)) {
              // extract the data for this event
              const [dispatchError] = event.data;
              let errorInfo;

              // decode the error
              if (dispatchError.isModule) {
                const decoded = api.registry.findMetaError(
                  dispatchError.asModule,
                );

                errorInfo = `${decoded.section}.${decoded.name}`;
              } else {
                errorInfo = dispatchError.toString();
              }

              let failed = `${section}.${method}:: ExtrinsicFailed:: ${errorInfo} at block ${i}`;

              console.log(failed);

              throw new Error(failed);
            }
          });
      },
    );
  }
}

main()
  .catch((err) => {
    console.error(err);
    process.exit(-1);
  })
  .then(() => process.exit(0));
