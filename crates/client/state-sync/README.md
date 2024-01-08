Madara state sync from l1 implementation.

## Workflow

1. It retrieves the synchronization starting point from the configuration file.
   It compares this information with the synchronization status saved in the
   local Madara backend (if it exists). It prioritizes the data in the Madara
   backend.
2. It creates a SyncOracle for other services to query the synchronization
   status.
3. It begins requesting state diff data from L1. Following Starknet's
   characteristics, it first retrieves the L2 state update block. Then, it
   fetches the fact of the state update block and queries the encoded state diff
   based on this fact. Therefore, if the contract is trusted, the state diff
   will also be trusted.
4. It decodes the encoded state diff. At this point, the state diff is not
   compatible with Madara because Madara is built on Substrate. Thus, it needs
   to convert the state diff into a Substrate-compatible format.
5. Utilizing the decoded storage changes, it constructs a new state and
   Substrate block based on the current state trie tree. It applies the new
   state and block to the Substrate application.
6. If the application is successful, it updates the L1 synchronization status in
   the Madara backend.
7. It monitors if synchronization has reached the highest block on L1. If so, it
   updates the status to 'Synced'. When the block builder detects that the
   current status is 'Synced', it can generate blocks at that time.
