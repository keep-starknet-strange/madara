# Cairo Contracts

This folder contains cairo contracts meant to be deployed to madara for
end-to-end testing.

## Installation

This is actually a python project using [poetry](https://python-poetry.org/) as
a package manager.

To install the project, make sure you have `poetry` available and run:

```bash
poetry install
```

## Usage

The goal of this folder is to provide artifacts for madara testing and deploy
scripts targeting madara endpoints for end-to-end testing.

First of all make sure you are inside the poetry env:

```bash
poetry shell
```

The script folder contains example scripts, for example for compiling all the
Cairo 0 contracts:

```bash
python scripts/compile_all.py
```

Or for deploying an ERC20

```bash
python scripts/deploy_erc20.py
```

## Class hash

Make sure you have `starkli` installed:

```bash
curl https://get.starkli.sh | sh
starkliup
```

In order to calculate class hash for Cairo 0 contract:

```bash
starkli class-hash path-to-output.json
```

For Cairo 1:

```bash
starkli class-hash path-to-output.casm.json
```
