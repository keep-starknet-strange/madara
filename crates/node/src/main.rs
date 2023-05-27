//! Madara node command line.
#![warn(missing_docs)]

#[macro_use]
mod service;
mod benchmarking;
mod chain_spec;
mod cli;
mod command;
mod rpc;
mod starknet;
mod constants;

fn main() -> sc_cli::Result<()> {
    command::run()
}
