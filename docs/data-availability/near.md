## NEAR Protocol as a Data Availability Layer

NEAR DA leverages an important part of NEAR's consensus mechanism, known as Nightshade, which parallelizes the network into multiple shards (essentially, multiple parallel blockchains). Each shard on NEAR produces a small portion of a block, which is called a chunk. These chunks are aggregated to produce blocks. All of this happens entirely at the protocol level and so is invisible to users and developers.

NEAR DA uses this infrastructure to an ETH rollup's benefit. When a chunk producer processes a receipt, there is consensus around the receipt. However, once the chunk has been processed and included in the block, the receipt is no longer required for consensus and can be pruned from the blockchain's state. The pruning time is at least 3 NEAR epochs, where each epoch is 12 hours. In practice, this is usually around 5 NEAR epochs, so data is available in the network for around 60 hours. Once the receipt is pruned, it is the responsibility of archival nodes to retain the transaction data.

This means that NEAR doesn't slow down its consensus with more data than it requires, yet any user of NEAR DA would have ample time to query transaction data. The advantage this architecture provides to rollups is cost-effective data availability, especially to those with high transaction volume, such as gaming chains.

### Building Madara with NEAR DA support

Madara can be compiled with support for NEAR as a DA layer via the `near` feature flag. For example,

```sh
cargo build --release --features=near
```

### Creating the daemon account

Depending on your setup, you may wish for the Madara client to use a separate NEAR account from that to which the blob store contract is deployed.

That statement may seem strange to Ethereum developers; the NEAR account model is a bit unique. A NEAR account address is distinct from (and not necessarily derived from) any public key. Instead, a NEAR account can have zero or more permissioned keys attached to it, as well as a maximum of one smart contract. This means that an account can be externally-owned (controlled by private key) _and_ a contract account at the _same time_.

It is possible to post blobs from the same account that has the contract deployed. Create a separate daemon account if you want.

### Deploying the NEAR DA smart contract

Because NEAR Protocol is a general-purpose layer-1 blockchain, it requires a [smart contract](https://github.com/near/rollup-data-availability/tree/main/contracts/blob-store) to be used as a data availability layer. The instructions for deploying this smart contract can be found [here](https://github.com/near/rollup-data-availability?tab=readme-ov-file#if-using-your-own-contract).

The basic steps are:
1. Build the contract.
2. Deploy the contract a NEAR address, for example `blob-store.near`.
3. Initialize the contract on chain by calling `new` on it. You _must_ call `new` from the daemon account, since the contract sets the owner to the predecessor account of this call, and only the contract owner may submit blobs. Ownership can be transferred using the `own_propose_owner` &rarr; `own_accept_owner` workflow.

You may find the following tools useful:
- [NEAR CLI](https://docs.near.org/tools/near-cli-rs) for general-purpose command-line interactions with the NEAR blockchain.

### Configure Madara

In you chain's base directory, create a file called `NEAR.json`. See [`examples/da-confs/near.json`](/examples/da-confs/near.json) for an example.

You will need:
- `account_id` - The account ID to post blobs _from_ (daemon-controlled).
- `secret_key` - An ED25519 secret key attached to the account specified in `account_id`. The key must have permission to call `submit` on the account specified in `contract_id`.
- `contract_id` - The account ID to post blobs _to_ (where the contract lives). `contract_id` may be the same as `account_id`.
- `network` - Either `Mainnet` or `Testnet`, depending on the NEAR network where the accounts live.
- `namespace` - Contains two numerical identifiers, `version` and `id`, which can be freely set by the user as a means of identifying the client that posted a blob.

### Run Madara

Run the Madara client with the `--da-layer=near` flag.

---

**🎉 Congratulations! 🎉**

You're using NEAR Protocol for data availability!

### Additional Resources

- [NEAR DA Repository](https://github.com/near/rollup-data-availability)
- [NEAR DA Documentation](https://docs.near.org/data-availability/welcome)
- [Official NEAR DA Announcement Post](https://near.org/blog/near-foundation-launches-near-da-to-offer-secure-cost-effective-data-availability-for-eth-rollups-and-ethereum-developers)
- [Why NEAR Data Availability? (blog post)](https://pages.near.org/blog/why-near-data-availability/)
- [Conference Talk](https://www.youtube.com/watch?v=GJOTSUeRKkk)
