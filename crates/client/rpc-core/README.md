# Starknet RPC-code

## Auto-generated types

The Starknet RPC is defined in the
[starkware-spec repo](https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_api_openrpc.json)
and follows the [Open RPC spec.](https://spec.open-rpc.org/)

The Open RPC organization also develops
[a tool](https://github.com/open-rpc/typings) to produce language-specific
typings given an OpenRPC document.

The `./src/types.rs` file has consequently been generated directly from the
Starkware spec using:

```bash
npx @open-rpc/typings \
  -d https://raw.githubusercontent.com/starkware-libs/starknet-specs/master/api/starknet_api_openrpc.json \
  --output-rs ./crates/client/rpc-core/src \
  --name-rs types
```

The generated file has then been manually fixed on a case by case basis.
