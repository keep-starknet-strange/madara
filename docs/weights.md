# Weights

Substrate uses weights to make sure extrinsics are not *too* large and actually bloating the execution.

> Substrate and FRAME provide a flexible framework for developing custom logic
> for your blockchain. This flexibility enables you to design complex and
> interactive pallets and implement sophisticated runtime logic. However,
> determining the appropriate weight to assign to the functions in your pallets
> can be a difficult task. Benchmarking enables you to measure the time it takes
> to execute different functions in the runtime and under different conditions.
> If you use benchmarking to assign accurate weights to function calls, you can
> prevent your blockchain from being overloaded and unable to produce blocks or
> vulnerable to denial of service (DoS) attacks by malicious actors.

You can learn more about it [here](https://docs.substrate.io/test/benchmark/).

## CairoVM and Weights

We need to map the cairo execution (steps/builtins) to substrate weights.
For this, we use a benchmarking script which you can find [here](../benchmarking/benchmarking.sh).

You can run it like this with different arguments.

If you want to get real **production** result then you should build madara with
production profile:

`cargo build --profile=production --features runtime-benchmarks`

```bash
chmod +x ./scripts/benchmarking.sh

./scripts/benchmarking.sh --help

./scripts/benchmarking.sh pallet_starknet infinite_loop
```
