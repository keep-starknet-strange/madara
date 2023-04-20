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
| starknet_getBlockWithTxHashes            | :construction:     |
| starknet_getBlockWithTxs                 | :construction:     |
| starknet_getStateUpdate                  | :construction:     |
| starknet_getStorageAt                    | :construction:     |
| starknet_getTransactionByHash            | :construction:     |
| starknet_getTransactionByBlockIdAndIndex | :construction:     |
| starknet_getTransactionReceipt           | :construction:     |
| starknet_getClass                        | :construction:     |
| starknet_getClassHashAt                  | :construction:     |
| starknet_getClassAt                      | :construction:     |
| starknet_getBlockTransactionCount        | :white_check_mark: |
| starknet_call                            | :construction:     |
| starknet_estimateFee                     | :construction:     |
| starknet_blockNumber                     | :white_check_mark: |
| starknet_blockHashAndNumber              | :white_check_mark: |
| starknet_chainId                         | :construction:     |
| starknet_pendingTransactions             | :construction:     |
| starknet_syncing                         | :construction:     |
| starknet_getEvents                       | :construction:     |
| starknet_getNonce                        | :construction:     |
| starknet_traceTransaction                | :construction:     |
| starknet_simulateTransaction             | :construction:     |
| starknet_traceBlockTransactions          | :construction:     |
| starknet_addInvokeTransaction            | :construction:     |
| starknet_addDeclareTransaction           | :construction:     |
| starknet_addDeployAccountTransaction     | :construction:     |

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
