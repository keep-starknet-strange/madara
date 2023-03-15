# Kaioshin Benchmarking

This is a collection of scripts and tools to benchmark Kaioshin leveraging artillery.

Steps to follow :
- Install the dependencies using `yarn install`
- Run the benchmark using `yarn test:ci`

The following benchmarks are available :
- `yarn test:chain` : Simple stresstest of the chain
- `yarn test:storage` : Deploys and execute cairo programs to benchmark the storage overhead
- `yarn test:execution` : Executes fib500 cairo programs


## References

Thank you to [https://github.com/dwellir-public/artillery-engine-substrate](artillery-substrate-engine) for the inspiration.
