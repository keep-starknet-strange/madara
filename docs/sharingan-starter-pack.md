# Sharingan starter pack

## Network information

- Release / tag:
  [v0.1.0-testnet-sharingan-beta.7.1](https://github.com/keep-starknet-strange/madara/releases/tag/v0.1.0-testnet-sharingan-beta.7.1)
- Docker image:
  [ghcr.io/keep-starknet-strange/madara:v0.1.0-testnet-sharingan-beta.7.1](https://github.com/keep-starknet-strange/madara/pkgs/container/madara)
- Bootnode:
  `/ip4/52.7.206.208/tcp/30333/p2p/12D3KooWK26CZBpWtwMaVQ6nXHrrXBkdXXx6CrBAU2KgLTqDNL6o`
- [Dev explorer](https://starknet-madara.netlify.app/?rpc=wss%3A%2F%2Fsharingan.madara.wtf%20#/explorer)

## Madara

[Madara](https://github.com/keep-starknet-strange/madara) is a Starknet
sequencer. Written using rust and substrate (a SDK to build blockchain), Madara
is a community driven sequencer supported by Starkware Keep-Starknet-Strange
team which is expected to be one of the major sequencers participating in the
Starknet decentralization.

If you don't know Madara, take a look at the repository and the
[contributor starter pack](https://github.com/keep-starknet-strange/madara/blob/main/docs/contributor-starter-pack.md)
to have an idea of how it is built.

However, if you just want to participate to Sharingan, you can continue reading
this guide, which will guide you without previous Madara knowledge.

## What is Sharingan

Sharingan is an ephemeral testnet for Starknet where all nodes participating in
the network are Madara instances. In this testnet, Starknet is being tested
decentralized, where the nodes work in consensus to determine which block is the
next to be added to the chain.

But there are also nodes that are participating in the data storage only,
without being involved in the consensus.

Even if Madara is called `Starknet Sequencer`, we distinguish two types of node
in Sharingan:

1. Madara as a `sequencer`, participating in the consensus.
2. Madara as a `fullnode`, for the data persistence.

For the rest of the guide, `sequencer` will refer to a Madara instance
participating to the consensus, and `fullnode` to a Madara instance used for
data persistence.

The objective of Sharingan is to start testing Starknet being decentralized and
also give an access to everybody to participate and test Starknet network.

To interact with the Starknet network, Starknet nodes expose a JSON RPC
endpoint. This means that any Madara instance participating in the network will
have an open port to allow external communication. More on this in the RPC
dedicated section.

If you speak spanish, you surely want to check
[this version](https://github.com/Nadai2010/Nadai-StarknetEs-Sharingan) of the
guide, made by Nadai for StarknetES community.

## Sharingan typology

As mentioned, Sharingan relies on `sequencers` to produce, validate and add
block to the chain. As of today, Sharingan has the following known `sequencers`:

<table>
  <tr>
    <th>Maintainer</th>
    <th>ID</th>
    <th>Key alias</th>
    <th>IP</th>
    <th>Peer ID</th>
    <th>RPC port</th>
  </tr>
  <tr>
    <td>Starkware</td>
    <td>1</td>
    <td><code>alice</code></td>
    <td><code>52.7.206.208</code></td>
    <td><code>12D3KooWK26CZBpWtwMaVQ6nXHrrXBkdXXx6CrBAU2KgLTqDNL6o</code></td>
    <td><code>9944</code></td>
  </tr>
  <tr>
    <td>Starkware</td>
    <td>2</td>
    <td><code>bob</code></td>
    <td><code>44.195.161.82</code></td>
    <td><code>12D3KooWMyrW5SvZ1WpcMLi7QdjXRQ36mUBg6RoaoABtPNMGVqXr</code></td>
    <td><code>9944</code></td>
  </tr>
  <tr>
    <td>Cartridge</td>
    <td>3</td>
    <td><code>charlie</code></td>
    <td><code>208.67.222.222</code></td>
    <td><code>--</code></td>
    <td><code>9944</code></td>
  </tr>
  <tr>
    <td>LambdaClass</td>
    <td>4</td>
    <td><code>dave</code></td>
    <td><code>65.108.65.148</code></td>
    <td><code>12D3KooWG29EKQvNoUHRWXwdNQ7LiEFG8wGS86CKyohe16sqYUM2</code></td>
    <td><code>9944</code></td>
  </tr>
  <tr>
    <td>Pragma</td>
    <td>5</td>
    <td><code>eve</code></td>
    <td><code>13.39.22.82</code></td>
    <td><code>12D3KooWMyJvL4qJZz9kTcD7xP2vBMqM9SMMFdJvD3tS9KEoMJCw</code></td>
    <td><code>9944</code></td>
  </tr>
  <tr>
    <td>Kakarot</td>
    <td>6</td>
    <td><code>ferdie</code></td>
    <td><code>52.50.242.182</code></td>
    <td><code>12D3KooWMzm38Uw32PS4aUxykX9vHWkG4TqKtYY6gqKSt6dwgmk3</code></td>
    <td><code>9944</code></td>
  </tr>
</table>

More technical details can be found in the discussion
[here](https://github.com/keep-starknet-strange/madara/discussions/553).

As Madara is using substrate, there is an existing web application that allows
you to monitor the Sharingan state. And to check the node typology, you can
directly go here to the
[node info](https://starknet-madara.netlify.app/?rpc=wss%3A%2F%2Fsharingan.madara.wtf%20#/explorer/node)
tab.

Currently, `sequencers` are chosen by Starkware, and you can join as a
`fullnode` only.

## Participate to Sharingan as a fullnode

To participate to Sharingan, there are few considerations to have in mind:

1. The Madara code you will be running will need to expose two ports: one for
   the peer-to-peer communication, which is currently `30333`, and one for the
   RPC mentioned earlier which is usually `9944`.

2. Substrate is using `chain specs` to have all participating node being in sync
   on how to collaborate on the network. The chain specification is also special
   in substrate, as it includes what we call the runtime, where we define how
   transactions are processed within the nodes.

3. Resources: to participate to Sharingan testnet as a fullnode, the hardware
   requirements are low and it will depend on how much you will query your
   fullnode.

   But basically, you can spin up a node with `2 vCPU` and `2GB RAM`, and Madara
   will run smoothly. We recommend `4GB RAM` as the initial synchronization of
   the blocks is more intensive, and can almost reach `2GB RAM`. Also if you
   have your node running a large amount of time the RAM will grow. A restart
   refreshes the RAM usage to `~500MB`.

   When Madara is on idle state, synchronizing the blocks at the head of the
   chain, it uses around `500MB/700MB RAM` and very few CPUs.

   Tested with `AMD EPYC 7000 series ~2.1GHz` and `Intel Xeon 3.3GHz`, with 2
   vCPU for the minimum configuration.

   If you are on AWS, `t2.medium` is a very good setup for a regular use if the
   instance is dedicated to Sharingan.

To participate to Sharingan as a fullnode, you have two options:

### Easy way: docker image

There is a docker image built for Sharingan, which will be updated at each
version of Sharingan. However, to ensure that you are using the correct chain
specs, please proceed to the following:

1. Create a dedicated directory to store sharingan data (in case you don't have
   one yet).

   ```bash
   mkdir sharingan-volume
   ```

2. Running the docker container.

   ```bash
   docker run --rm -d \
    -p 9944:9944 -p 30333:30333 \
    --name sharingan-fullnode \
    -v sharingan-volume:/root/.madara \
    ghcr.io/keep-starknet-strange/madara:v0.1.0-testnet-sharingan-beta.7.1 \
    --testnet sharingan
   ```

Consider running the node in detached mode using the `-d` option. But try first
running without the `-d` option as it's easier to see what's happening if it's
your first time using docker. Alternatively, you can also run
`docker logs -f sharingan-fullnode`.

### Dev way: cloning Madara repository

If you prefer having Madara compiled locally, you must:

1. Clone [Madara repository](https://github.com/keep-starknet-strange/madara).
2. Checkout on the tag `v0.1.0-testnet-sharingan-beta.7.1`.
3. `cargo build --workspace --release` (you can check
   [this guide](https://github.com/keep-starknet-strange/madara/blob/main/docs/rpc-contribution.md)
   with some info about compiling Madara).
4. Run the fullnode (in a screen, or any other mean to keep it running) being at
   the root of Madara repository:

```bash
./target/release/madara --testnet sharingan
```

This will store the data into `$HOME/.madara`.

Once you have your node running, you can get your Peer ID running the following
command:

```bash
curl --header "Content-Type: application/json" \
  --request POST \
  --data '{
    "jsonrpc": "2.0",
    "method": "system_localPeerId",
    "params": [],
    "id": 1
}' \
http://127.0.0.1:9944
```

You can then go to the
[node explorer](https://starknet-madara.netlify.app/?rpc=wss%3A%2F%2Fsharingan.madara.wtf%20#/explorer/node)
and check that your node is appearing and start synchronizing the blocks.

**Welcome to Sharingan, you're in!** :rocket:

For now, there are few transactions, and the storage for the 20,000+ blocks is
less that 1GB. But this will vary in the future depending on the network
activity.

## Interact with Starknet using JSON RPC

To interact with Starknet using a node of Sharingan, you must use JSON RPC
directly (with command like `curl`) or indirectly using Starknet CLI programs
like [starkli](https://github.com/xJonathanLEI/starkli) (Some documentation
coming soon for starkli).

Currently, Madara is still under active development and it's recommended to
regularly check the
[Starknet features compatibility page of Madara](https://github.com/keep-starknet-strange/madara/blob/v0.1.0-testnet-sharingan-beta.7.1/docs/starknet_features_compatibility.md).

Here are some examples of RPC using the sequencer-1 of Starkware:

### Get the last block hash

```bash
starkli block-hash --rpc http://52.7.206.208:9944
```

```bash
curl --header "Content-Type: application/json" \
     --request POST \
     --data '{
     "jsonrpc": "2.0",
     "method": "starknet_blockHashAndNumber",
     "params": ["latest"],
     "id":1}' \
     http://52.7.206.208:9944
```

## Sharingan substrate explorer

[Here](https://starknet-madara.netlify.app/?rpc=wss%3A%2F%2Fsharingan.madara.wtf%20#/explorer/node)
is the web app to visualize Sharingan status.

As mentioned earlier, Madara is using substrate SDK. As a particularity of how
Madara uses substrate, any Starknet block is produced and wrapped into a
substrate block.

Substrate blocks are not using the same hashing method that Starknet uses. For
this reason, the blocks visible in the explorer are not matching the blocks you
can see in Starknet explorer like starkscan or voyager.

But this is a great tool to check Sharingan typology and block production.

## What can I do on Sharingan

For now, Madara (and then Sharingan) is not supporting Cairo 1, this is coming
soon!

The idea is to have lots of people using Sharingan as a testnet, deploying and
using contracts sending transactions to Sharingan nodes.

This will be the most effective way to initiate Starknet decentralization.

For now you can take existing Cairo 0 contracts and play with them on Sharingan,
but be prepared.. Quantum Leap is coming and Madara is not far behind...!
:rocket:

Let's continue building and thank you for being part or having interest for
Sharingan testnet!
