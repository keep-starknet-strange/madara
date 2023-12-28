pub mod client;
pub mod node;

use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use derive_more::Display;
use lazy_static::lazy_static;
use tokio::sync::Mutex;

use crate::client::MadaraClient;
use crate::node::MadaraNode;

lazy_static! {
    /// This is to prevent TOCTOU errors; i.e. one background madara node might find one
    /// port to be free, and while it's trying to start listening to it, another instance
    /// finds that it's free and tries occupying it
    /// Using the mutex in `get_free_port_listener` might be safer than using no mutex at all,
    /// but not sufficiently safe
    static ref FREE_PORT_ATTRIBUTION_MUTEX: Mutex<()> = Mutex::new(());
}

#[derive(Display, Clone)]
pub enum Settlement {
    Ethereum,
}

pub struct MadaraRunner {
    _node: MadaraNode,
    client: MadaraClient,
}

impl MadaraRunner {
    pub async fn new(settlement: Option<Settlement>, base_path: Option<PathBuf>) -> Self {
        // we keep the reference, otherwise the mutex unlocks immediately
        let _mutex_guard = FREE_PORT_ATTRIBUTION_MUTEX.lock().await;

        let mut node = MadaraNode::run(settlement, base_path);
        let client = MadaraClient::new(node.url());

        // Wait until node is ready
        loop {
            // Check if there are no build / launch issues
            if let Some(status) = node.has_exited() {
                panic!("Madara node exited early with {}", status)
            }

            match client.health().await {
                Ok(is_ready) if is_ready => break,
                _ => {}
            }
        }

        Self { _node: node, client }
    }
}

impl Deref for MadaraRunner {
    type Target = MadaraClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for MadaraRunner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
