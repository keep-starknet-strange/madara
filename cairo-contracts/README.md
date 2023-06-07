# Cairo Contracts

This folder contains cairo contracts meant to be deployed to madara for
end-to-end testing.

## Installation

This is actually a python project using [poetry](https://python-poetry.org/) as
a package manage.

To install the project, make sure you have `poetry` available and run:

```bash
poetry install
```

## Usage

The goal of this folder is to provide artifacts for madara testing and deploy
scripts targeting madara endpoints for end-to-end testing.

Every cairo file in the `src` is automatically compiled when running

```bash
python utils/compile_all.py
```

An example deploy script
