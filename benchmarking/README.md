# Madara Benchmarking

This is a collection of scripts and tools to benchmark Madara leveraging
artillery.

Steps to follow :

- Install the dependencies using
  `yarn install && cd ../tests && npm install && npm run build && cd ../benchmarking`
- Make sure you've built the project using `cd .. && cargo build --release`
- Run the benchmark using `yarn test:ci`. If it does not work, just run
  `cd .. && sh ./scripts/run_node.sh` and in another terminal run `yarn test:x`
  where x is the benchmark you want to run.

The following benchmarks are available :

- `yarn test:chain` : Simple stresstest of the chain
- `yarn test:storage` : Deploys and execute cairo programs to benchmark the
  storage overhead
- `yarn test:execution` : Executes fib500 cairo programs
- `yarn test:transfer` : Executes ERC20 transfers

Or simply run `yarn test` to run default benchmark and display metrics at the
end.

## References

Thank you to
[https://github.com/dwellir-public/artillery-engine-substrate](artillery-substrate-engine)
for the inspiration.
