# Madara <> Celestia Integration

This hackathon project allows for the posting of Madara State Diffs to the
Celestia Data Availability layer.

### Setup

This project consists of 2 main components:

- A running Madara node
- A running Celestia light node OR a celestia RPC url and credentials to post to
  this RPC

### To get our Celestia light node running and set up

Requires the installation of the celestia CLI tool (available
[here](https://docs.celestia.org/developers/node-tutorial/)). Once you have that
set up, run the following:

```
celestia light start --core.ip consensus-full-arabica-9.celestia-arabica.com --gateway.deprecated-endpoints --gateway --gateway.addr 127.0.0.1 --gateway.port 26659 --p2p.network arabica-9
```

Next, we need to generate a JWT to access our light node:

```
celestia light auth admin --p2p.network arabica-9
```

Apologies, but as this is hacky you'll have to save the output of this as `jwt`
in `crates/client/data-availability/src/celestia/mod.rs`

### To get our Madara node running

From root, run the following:

```
cargo run --release -- --dev --l1-node celestia-rpc-address
```

### Testing the Node

Once we have our node running - we can run a test to create a state diff:

from root - to set up the suite:

```
cd benchmarking && npm install && cd ../tests && npm install && npm run build && cd ../benchmarking
```

and to run the test:

```
npm run test:transfer
```

### Files of Note/What is happening here

This implementation is basically just taking the Madara node software and
swapping out where it saves to storage with a call to the celestia network
