# Madara Benchmarking

This is a collection of scripts and tools to benchmark Madara leveraging artillery.

Start by installing dependencies using `yarn install`.

## Running scripts
- `yarn script:transfer` : Declare and deploy an ERC20 contract and transfer tokens between accounts


## Running the benchmarks

Steps to follow :
- Make sure you've built the project using `cd .. && cargo build release`
- Run the benchmark using `yarn test:ci`.
  If it does not work, just run `sh ../scripts/run_node.sh` and in another terminal run `yarn test` where x is the benchmark you want to run.

The following benchmarks are available :
- `yarn test:chain` : Simple stresstest of the chain
- `yarn test:storage` : Deploys and execute cairo programs to benchmark the storage overhead
- `yarn test:execution` : Executes fib500 cairo programs
- `yarn test:transfer` : Executes starknet ERC20 `transfer` transactions


## References

Thank you to [https://github.com/dwellir-public/artillery-engine-substrate](artillery-substrate-engine) for the inspiration.
