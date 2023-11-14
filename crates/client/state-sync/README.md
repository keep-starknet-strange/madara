Madara state sync from l1 implementation.

issue: https://github.com/keep-starknet-strange/madara/issues/1224

## Syncing

The implementation will involve(w/ python examples):

- SN Genesis defs
- from genesis fetch the state update logs and the state transition fact logs
- fetch the memory page logs for that block(or in bulk)
- Fetch all LogMemoryPageFactContinuous events from the MemoryPageFactRegistry
- decode the function input tied to this memory page fact which will correspond to the encoded DA output described above
- parse the DA and apply the relevant changes to your aggregated state via mapping sync
- repeat until you reach to current head

