# Data Availability Testing

To contribute to the DA related crates, you will need to run the tests locally.

## Run tests

First you will need to run locally the DA devnet node.

```bash
bash scripts/da_devnet.sh <da_layer>
```

Once it's up and running, you can run madara with the same DA layer.

```bash
./target/release/madara --dev --da-layer <da_layer> --da-conf examples/da-confs/<da_layer>.json
```

Now you can run the tests inside the `da-test` crate.

```bash
cd da-test
DA_LAYER=<da_layer> cargo test
```

Finally make sure to stop the DA devnet node.

```bash
bash scripts/stop_da_devnet.sh <da_layer>
```
