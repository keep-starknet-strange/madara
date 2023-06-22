# StarkNet Features Compatibility

## Block

| Feature                | State              |
| ---------------------- | ------------------ |
| Parent block hash      | :white_check_mark: |
| Block number           | :white_check_mark: |
| Global state root      | :construction:     |
| Sequencer address      | :construction:     |
| Block timestamp        | :white_check_mark: |
| Transaction count      | :white_check_mark: |
| Transaction commitment | :white_check_mark: |
| Event count            | :white_check_mark: |
| Event commitment       | :white_check_mark: |
| Protocol version       | :white_check_mark: |
| Extra data             | :white_check_mark: |

## Transaction

| Feature    | State              |
| ---------- | ------------------ |
| Declare    | :white_check_mark: |
| Deploy     | :white_check_mark: |
| Invoke     | :white_check_mark: |
| L1 Handler | :construction:     |

## RPC

| Feature                                  | State              |
| ---------------------------------------- | ------------------ |
| starknet_getBlockWithTxHashes            | :white_check_mark: |
| starknet_getBlockWithTxs                 | :white_check_mark: |
| starknet_getStateUpdate                  | :white_check_mark: |
| starknet_getStorageAt                    | :white_check_mark: |
| starknet_getTransactionByHash            | :white_check_mark: |
| starknet_getTransactionByBlockIdAndIndex | :white_check_mark: |
| starknet_getTransactionReceipt           | :white_check_mark: |
| starknet_getClass                        | :white_check_mark: |
| starknet_getClassHashAt                  | :white_check_mark: |
| starknet_getClassAt                      | :white_check_mark: |
| starknet_getBlockTransactionCount        | :white_check_mark: |
| starknet_call                            | :white_check_mark: |
| starknet_estimateFee                     | :white_check_mark: |
| starknet_blockNumber                     | :white_check_mark: |
| starknet_blockHashAndNumber              | :white_check_mark: |
| starknet_chainId                         | :white_check_mark: |
| starknet_pendingTransactions             | :white_check_mark: |
| starknet_syncing                         | :white_check_mark: |
| starknet_getEvents                       | :white_check_mark: |
| starknet_getNonce                        | :white_check_mark: |
| starknet_traceTransaction                | :construction:     |
| starknet_simulateTransaction             | :construction:     |
| starknet_traceBlockTransactions          | :construction:     |
| starknet_addInvokeTransaction            | :white_check_mark: |
| starknet_addDeclareTransaction           | :white_check_mark: |
| starknet_addDeployAccountTransaction     | :white_check_mark: |

## Decentralisation

| Feature                                | State              |
| -------------------------------------- | ------------------ |
| Single node                            | :white_check_mark: |
| Small pool of nodes (POA)              | :construction:     |
| Large pool of nodes (Base consensus)   | :construction:     |
| Large pool of nodes (Custom consensus) | :construction:     |

## Optimisation

| Feature                             | State          |
| ----------------------------------- | -------------- |
| Commitments                         | :construction: |
| Transaction validity before mempool | :construction: |
