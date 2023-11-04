#![allow(deprecated)]

mod convert;
mod fetch;
mod utility;
#[cfg(feature = "m")]
mod m;

pub use fetch::BlockFetchConfig;

type CommandSink = futures_channel::mpsc::Sender<sc_consensus_manual_seal::rpc::EngineCommand<sp_core::H256>>;

pub async fn fetch_block(command_sink: CommandSink, sender: tokio::sync::mpsc::Sender<mp_block::Block>, fetch_config: BlockFetchConfig, rpc_port: u16) {
    #[cfg(feature = "m")]
    {
        if fetch_config.sound {
            m::init();
        }
    }
    
    let first_block = utility::get_last_synced_block(rpc_port).await + 1;
    fetch::fetch_blocks(command_sink, sender, fetch_config, first_block).await;
}
