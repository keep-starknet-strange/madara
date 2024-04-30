# Starknet block import

This crate introduces a Starknet specific handler that can be added to the block
import pipeline. More specifically, it "wraps" the underlying block import logic
i.e. executes first in the queue.

Read more about block import pipeline:

- <https://docs.substrate.io/learn/transaction-lifecycle/#block-authoring-and-block-imports>
- <https://doc.deepernetwork.org/v3/advanced/block-import/>

The purpose of this custom block import logic is to do additional checks for
declare transactions. Despite the introduction of the safe intermediate
representation (Sierra) for Cairo, there's still a possibility of the following
attack:

- User crafts a malicious Cairo contract the execution of which cannot be proven
- User submits that transaction via a patched node that does not verify Sierra
  classes
- The runtime does not have Sierra classes and therefore cannot check the
  validiy either
- Sequencer fails to prove the execution and also cannot charge for the work
  done => DoS attack vector

Read more about Sierra and the problem it addresses:

- <https://docs.starknet.io/documentation/architecture_and_concepts/Smart_Contracts/cairo-and-sierra/>

Starknet block import solves the issue above as follows:

- Upon receiving a new block it searches for declare transactions
- For every declare transaction found, it tries to find according Sierra classes
  in the local DB
- It tries to compile the class to check if they match the contract class from
  the transaction
- The block import fails if there is at least one transaction with mismatching
  class hashes

## Notes

Currently Sierra classes DB is populated when user submits RPC requests and
therefore the approch works for single node setup only. In order to make it work
in the multi-node setting, one needs to query missing Sierra classes from other
nodes via P2P (see
<https://github.com/starknet-io/starknet-p2p-specs/blob/main/p2p/proto/class.proto>)

Cairo compiler version mismatch can be a problem, e.g. if the version used by
Madara is lagging behind significantly it can fail to compile Sierra classes
obtained from the recent compilers. Similarly, old Sierra classes might not
compile because of the broken backward compatibility.
