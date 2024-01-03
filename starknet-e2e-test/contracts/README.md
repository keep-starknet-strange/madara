# Solidity contracts

This folder contains compiled contracts that are used for integration / e2e
tests of Madara L1 <> L2 messaging.

## Compilation

First make sure you have
[Foundry](https://book.getfoundry.sh/getting-started/installation) installed.

Starknet contract sources live in [madara-zaun](../madara-zaun/) folder which is
git submodule pointing at
[zaun repo](https://github.com/keep-starknet-strange/zaun). If you haven't
cloned the submodules together with the Madara repository, run:

```bash
git submodule update --init
```

If `zaun` submodule is out of sync, do:

```bash
git submodule sync --recursive
```

In this folder run:

```bash
make artifacts
```
