## Madara Faucet

You can have a custom faucet for your app chains!

<figure class="video_container">
  <video controls="true" allowfullscreen="true" poster="videos/starkcet_demo.mp4">
    <source src="videos/starkcet_demo.mp4" type="video/mp4">
  </video>
</figure>

### Setting up your Madara faucet

Follow the steps below to setup a faucet for your local build

### Madara Run

Run an instance of your madara node locally

```bash
cargo run --release -- setup
cargo run --release -- run --dev
```

`--dev`: enforces a development environment needed to make testing easier for
your node. If you're running this without `--dev`, make sure to add

1. `--rpc-cors="*"` or `--rpc-cors="<backend_origin>"` to allow the backend to
   communicate with your node
2. `--force-authoring` if you're the only node on your chain. This flag forces
   Madara to create blocks even if they are no peers.

### Faucet backend and frontend

Madara has currently integrated the Starkcet faucet which provides an easy to
use frontend and backend for your local build. You can start the frontend and
backend using Docker like this

```bash
git clone https://github.com/keep-starknet-strange/madara-infra
cd madara-infra/starknet-stack/
docker-compose up -d starkcet-front starkcet-back
```

Use `docker ps` to check if your containers are running. You should find two
containers with the following names

1. `starknet-stack-starkcet-front-1`
2. `starknet-stack-starkcet-back-1`

If you see these containers, congrats, your faucet is now running 🎉

### Interacting with your faucet

1. Go to <http://localhost:3000>
2. Enter your wallet address
3. Click Get Tokens
4. Done!

If you're clicking on `Get Tokens` repeatedly, ensure that the previous block
was added so that the new request to get tokens is sent with the correct nonce.

### Building your own faucet

If you have a use case where you need to customize your faucet or you need to
get faucet funds using code, you can achieve this by simply transferring funds
from any of the genesis accounts using RPC calls. The genesis account private
key for address `0x2` is available in
`crates/pallets/starknet/src/tests/constants.rs`.

Keep in mind that account `0x1` on Madara doesn't support multicall so
`account.execute` from starknetjs fails. You can either invoke the transfer
transaction as shown
[here](https://github.com/keep-starknet-strange/madara/blob/c916046adf9d7ea52131442090fae654ba6b234d/tests/util/starknet.ts#L241)
or use an account like `0x2` which is based on Argent and supports multicall.

**Example code for collecting tokens from `0x2` using starknetjs**

```javascript
import * as starknet from "starknet";

const eth_address =
  "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";
const provider = new starknet.RpcProvider({
  nodeUrl: "http://localhost:9944",
});
const starkKeyPair = starknet.ec.getKeyPair(
  "0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d",
);
const address = "0x2";

async function transfer(to) {
  const nonce = await provider.getNonceForAddress(address);
  const chainId = await provider.getChainId();

  const calldata = starknet.transaction.fromCallsToExecuteCalldata([
    {
      contractAddress: eth_address,
      entrypoint: "transfer",
      calldata: starknet.stark.compileCalldata({
        recipient: to,
        amount: {
          type: "struct",
          low: "1000000",
          high: "0",
        },
      }),
    },
  ]);
  const maxFee = "0x11111111111";
  const version = "0x1";
  const txnHash = starknet.hash.calculateTransactionHash(
    address,
    version,
    calldata,
    maxFee,
    chainId,
    nonce,
  );
  const signature = starknet.ec.sign(starkKeyPair, txnHash);
  const invocationCall = {
    signature,
    contractAddress: address,
    calldata,
  };
  const invocationDetails = {
    maxFee,
    nonce,
    version,
  };

  // if estimating fees passes without failures, the txn should go through
  const estimateFee = await provider.getEstimateFee(
    invocationCall,
    invocationDetails,
  );
  console.log("Estimate fee - ", estimateFee);

  const tx = await provider.invokeFunction(invocationCall, invocationDetails);
  console.log(tx.transaction_hash);
}

transfer("0x11");
```
