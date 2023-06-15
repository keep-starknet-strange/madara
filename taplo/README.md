# Taplo

[Taplo](https://github.com/tamasfe/taplo) is a TOML validator and formatter. It
provides a command-line interface (CLI) for working with TOML files.

## Installation

You can install Taplo using either cargo or Yarn or NPM.

### Cargo

```bash
cargo install taplo-cli --locked
```

### Yarn

```bash
yarn global add @taplo/cli
```

### NPM

```bash
npm install -g @taplo/cli
```

### Usage

To check your TOML files for formatting issues, use the following command:

```bash
npx @taplo/cli fmt --config taplo.toml --check
```

To format all TOML files in your project, use the following command:

```bash
npx @taplo/cli fmt --config taplo.toml
```

This command will automatically format the TOML files, ensuring consistent and
readable formatting.

### Configuration

Taplo allows you to customize the formatting rules by adding configuration
options. You can find the available options and how to use them
[here](https://taplo.tamasfe.dev/configuration/formatter-options.html).
