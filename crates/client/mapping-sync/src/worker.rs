use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

// Frontier
use futures::prelude::*;
use futures::task::{Context, Poll};
use futures_timer::Delay;
use log::debug;
use mc_storage::OverrideHandle;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
// Substrate
use sc_client_api::{
    backend::{Backend, StorageProvider},
    client::ImportNotifications,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SyncStrategy {
    Normal,
    Parachain,
}

pub struct MappingSyncWorker<B: BlockT, C, BE> {
    import_notifications: ImportNotifications<B>,
    timeout: Duration,
    inner_delay: Option<Delay>,

    client: Arc<C>,
    substrate_backend: Arc<BE>,
    overrides: Arc<OverrideHandle<B>>,
    madara_backend: Arc<madara_db::Backend<B>>,

    have_next: bool,
    retry_times: usize,
    sync_from: <B::Header as HeaderT>::Number,
    strategy: SyncStrategy,
}

impl<B: BlockT, C, BE> Unpin for MappingSyncWorker<B, C, BE> {}

impl<B: BlockT, C, BE> MappingSyncWorker<B, C, BE> {
    pub fn new(
        import_notifications: ImportNotifications<B>,
        timeout: Duration,
        client: Arc<C>,
        substrate_backend: Arc<BE>,
        overrides: Arc<OverrideHandle<B>>,
        frontier_backend: Arc<madara_db::Backend<B>>,
        retry_times: usize,
        sync_from: <B::Header as HeaderT>::Number,
        strategy: SyncStrategy,
    ) -> Self {
        Self {
            import_notifications,
            timeout,
            inner_delay: None,

            client,
            substrate_backend,
            overrides,
            madara_backend: frontier_backend,

            have_next: true,
            retry_times,
            sync_from,
            strategy,
        }
    }
}

impl<Block: BlockT, C, BE> Stream for MappingSyncWorker<Block, C, BE>
where
    C: ProvideRuntimeApi<Block>,
    C::Api: StarknetRuntimeApi<Block>,
    C: HeaderBackend<Block> + StorageProvider<Block, BE>,
    BE: Backend<Block>,
{
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<()>> {
        let mut fire = false;

        loop {
            match Stream::poll_next(Pin::new(&mut self.import_notifications), cx) {
                Poll::Pending => break,
                Poll::Ready(Some(_)) => {
                    fire = true;
                }
                Poll::Ready(None) => return Poll::Ready(None),
            }
        }

        let timeout = self.timeout;
        let inner_delay = self.inner_delay.get_or_insert_with(|| Delay::new(timeout));

        match Future::poll(Pin::new(inner_delay), cx) {
            Poll::Pending => (),
            Poll::Ready(()) => {
                fire = true;
            }
        }

        if self.have_next {
            fire = true;
        }

        if fire {
            self.inner_delay = None;

            match crate::sync_blocks(
                self.client.as_ref(),
                self.substrate_backend.as_ref(),
                self.overrides.clone(),
                self.madara_backend.as_ref(),
                self.retry_times,
                self.sync_from,
                self.strategy,
            ) {
                Ok(have_next) => {
                    self.have_next = have_next;
                    Poll::Ready(Some(()))
                }
                Err(e) => {
                    self.have_next = false;
                    debug!(target: "mapping-sync", "Syncing failed with error {:?}, retrying.", e);
                    Poll::Ready(Some(()))
                }
            }
        } else {
            Poll::Pending
        }
    }
}
