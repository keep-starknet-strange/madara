# Configurations

Madara has multiple custom configurations that change the behavior of some Substrate flags, enable settings by default and enable the customization and extension for app chains.

## Custom flags
- `--base-path`: Alias to `--madara-path`
- `--chain-spec-url`: Define an url to retrieve and load a custom chain spec
- `--configs-url`: Define an url to fetch a Madara configs index file
- `--dev`: We remapped the dev flag because it was too hard to reproduce the default Substrate dev environment. This flags sets up the following flags:
  - `--chain dev`
  - `--force-authoring`
  - `--alice`
  - `--rpc-external`
  - `--rpc-methods unsafe`
  - We need to enable manually the flag `--rpc-cors all` to accept external connections
- `--disable-url-fetch`: Disable the automatic url fetching that we use to retrieve configuration files
- `--madara-path`: Changes the default [madara-path](#madara-path) (default: `$HOME/.madara`)
- `--node-key-file`: Key responsible for assigning the peer_id (default: `madara-path/p2p-key.ed25519`)
- `--testnet`: Allows you to join an official testnet, currently only Sharingan is supported
- `--update-configs`: By default, if configuration files are already downloaded into `--madara-path`, the client will not try to overwrite them. With this flag, you can force an update

## Madara Path
All the Madara configurations, databases, and relevant files, live in the `madara-path`. The file system has the following folder structure:
- `chain-specs`: Folder with all the chain specs, downloaded through `--chain-spec-url` or `--testnet`
- `chains`: Databases with the different chains that were run on your computer
- `configs`: Configs fetched either by the filesystem (cargo project path) or remotely by using `--configs-url` or automatically (from the official repository)
  - `genesis-assets`: All the assets that will be loaded on the genesis. It is mandatory to have a `genesis.json` file in this path 
  - `index.json`: An index file that has all the official `chain-specs` and `configs`. You can extend this file and load it with the flag `--configs-url`
- `p2p-key.ed25519`: P2P key file
