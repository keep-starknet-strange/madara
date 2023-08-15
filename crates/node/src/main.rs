//! Madara node command line.
#![warn(missing_docs)]

#[macro_use]
mod service;
mod benchmarking;
mod chain_spec;
mod cli;
mod command;
mod constants;
mod genesis_block;
mod rpc;
mod starknet;

fn main() -> sc_cli::Result<()> {
    command::run()
}
