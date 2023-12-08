//! A worker syncing the Madara db
//!
//! # Role
//! The `MappingSyncWorker` listen to new Substrate blocks and read their digest to find
//! `pallet-starknet` logs. Those logs should contain the data necessary to update the Madara
//! mapping db: a starknet block header.
//!
//! # Usage
//! The madara node should spawn a `MappingSyncWorker` among it's services.

mod sync_blocks;

use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::prelude::*;
use futures::task::{Context, Poll};
use futures_timer::Delay;
use log::debug;
use mp_hashers::HasherT;
use pallet_starknet_runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::{Backend, StorageProvider};
use sc_client_api::client::ImportNotifications;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};

/// The worker in charge of syncing the Madara db when it receive a new Substrate block
pub struct MappingSyncWorker<B: BlockT, C, BE, H> {
    import_notifications: ImportNotifications<B>,
    timeout: Duration,
    inner_delay: Option<Delay>,

    client: Arc<C>,
    substrate_backend: Arc<BE>,
    madara_backend: Arc<mc_db::Backend<B>>,
    hasher: PhantomData<H>,

    have_next: bool,
    retry_times: usize,
    sync_from: <B::Header as HeaderT>::Number,
}

impl<B: BlockT, C, BE, H> Unpin for MappingSyncWorker<B, C, BE, H> {}

#[allow(clippy::too_many_arguments)]
impl<B: BlockT, C, BE, H> MappingSyncWorker<B, C, BE, H> {
    pub fn new(
        import_notifications: ImportNotifications<B>,
        timeout: Duration,
        client: Arc<C>,
        substrate_backend: Arc<BE>,
        frontier_backend: Arc<mc_db::Backend<B>>,
        retry_times: usize,
        sync_from: <B::Header as HeaderT>::Number,
    ) -> Self {
        Self {
            import_notifications,
            timeout,
            inner_delay: None,

            client,
            substrate_backend,
            madara_backend: frontier_backend,
            hasher: PhantomData,

            have_next: true,
            retry_times,
            sync_from,
        }
    }
}

impl<B: BlockT, C, BE, H> Stream for MappingSyncWorker<B, C, BE, H>
where
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    C: HeaderBackend<B> + StorageProvider<B, BE>,
    BE: Backend<B>,
    H: HasherT,
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

            match sync_blocks::sync_blocks::<_, _, _, H>(
                self.client.as_ref(),
                self.substrate_backend.as_ref(),
                self.madara_backend.as_ref(),
                self.retry_times,
                self.sync_from,
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
