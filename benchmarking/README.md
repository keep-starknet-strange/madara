# Madara Benchmarking

This is a collection of scripts and tools to benchmark Madara leveraging
artillery.

Steps to follow :

- Install the dependencies using
  `npm install && cd ../tests && npm install && npm run build && cd ../benchmarking`
- Make sure you've built the project using `cd .. && cargo build --release`
- Run the benchmark using `npm run test:ci`. If it does not work, just run
  `cd .. && sh ./scripts/run_node.sh` and in another terminal run
  `npm run test:x` where x is the benchmark you want to run.

The following benchmarks are available :

- `npm run test:chain` : Simple stress test of the chain
- `npm run test:storage` : Deploys and execute cairo programs to benchmark the
  storage overhead
- `npm run test:execution` : Executes fib500 cairo programs
- `npm run test:transfer` : Executes ERC20 transfers

Or simply run `npm run test` to run default benchmark and display metrics at the
end.

## References

Thank you to
[https://github.com/dwellir-public/artillery-engine-substrate](artillery-substrate-engine)
for the inspiration.
