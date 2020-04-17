pub mod sync_messages;

use actix::Addr;
use anyhow::Result;
use dyn_clone::{clone_box, DynClone};
use futures::executor::block_on;
use parking_lot::RwLock;
use starcoin_bus::{Broadcast, BusActor};
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::block::BlockNumber;
use starcoin_types::system_events::SystemEvents;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait StateSyncReset: DynClone + Send + Sync {
    async fn reset(&self, root: HashValue);
}

#[derive(Clone)]
pub struct SyncMetadata(Arc<RwLock<SyncMetadataInner>>);

pub struct SyncMetadataInner {
    is_state_sync: bool,
    pivot_behind: Option<(BlockNumber, u64)>,
    state_sync_address: Option<Box<dyn StateSyncReset>>,
    state_sync_done: bool,
    block_sync_done: bool,
    bus: Addr<BusActor>,
}

impl SyncMetadata {
    pub fn new(config: Arc<NodeConfig>, bus: Addr<BusActor>) -> SyncMetadata {
        info!("is_state_sync : {}", config.sync.is_state_sync());
        let inner = SyncMetadataInner {
            is_state_sync: config.sync.is_state_sync(),
            pivot_behind: None,
            state_sync_address: None,
            state_sync_done: false,
            block_sync_done: false,
            bus,
        };
        SyncMetadata(Arc::new(RwLock::new(inner)))
    }

    pub fn state_syncing(&self) -> bool {
        self.state_sync_mode() && !self.0.read().state_sync_done && !self.0.read().block_sync_done
    }

    pub fn update_pivot(&self, pivot: BlockNumber, behind: u64) -> Result<()> {
        assert!(self.state_syncing(), "cat not update pivot.");
        assert!(pivot > 0, "pivot must be positive integer.");
        assert!(behind > 0, "behind must be positive integer.");
        self.0.write().pivot_behind = Some((pivot, behind));
        Ok(())
    }

    pub fn state_sync_done(&self) -> Result<()> {
        assert!(self.state_sync_mode(), "chain is not in fast sync mode.");
        assert!(!self.0.read().state_sync_done, "state sync already done.");
        let mut lock = self.0.write();
        lock.state_sync_done = true;
        drop(lock);
        let _ = self.sync_done();
        info!("state sync done.");
        Ok(())
    }

    pub fn block_sync_done(&self) -> Result<()> {
        if !self.0.read().block_sync_done {
            let mut lock = self.0.write();
            lock.state_sync_done = true;
            let _ = self.sync_done();
            info!("block sync done.");
        }
        Ok(())
    }

    pub fn is_sync_done(&self) -> bool {
        (self.state_sync_mode() && self.0.read().state_sync_done && self.0.read().block_sync_done)
            || (!self.state_sync_mode() && self.0.read().block_sync_done)
    }

    fn sync_done(&self) -> Result<()> {
        if self.is_sync_done() {
            let mut lock = self.0.write();
            lock.pivot_behind = None;
            lock.state_sync_address = None;
            let bus = lock.bus.clone();
            block_on(async move {
                let _ = bus
                    .send(Broadcast {
                        msg: SystemEvents::SyncDone(),
                    })
                    .await;
            });
            info!("state sync and block sync done.");
        }
        Ok(())
    }

    pub fn state_sync_mode(&self) -> bool {
        self.0.read().is_state_sync
    }

    pub fn get_pivot(&self) -> Result<Option<BlockNumber>> {
        Ok(match self.0.read().pivot_behind.clone() {
            None => None,
            Some((pivot, _behind)) => Some(pivot),
        })
    }

    pub fn get_latest(&self) -> Option<BlockNumber> {
        match self.0.read().pivot_behind.clone() {
            None => None,
            Some((pivot, behind)) => Some(pivot + behind),
        }
    }

    pub fn update_address(&self, address: &(dyn StateSyncReset + 'static)) -> Result<()> {
        assert!(self.state_syncing(), "cat not update address.");
        self.0.write().state_sync_address = Some(clone_box(address));
        Ok(())
    }

    pub fn get_address(&self) -> Option<Box<dyn StateSyncReset>> {
        let lock = self.0.read();
        if let Some(ssr_ref) = lock.state_sync_address.as_deref() {
            let ssr_box = clone_box(ssr_ref);
            Some(ssr_box)
        } else {
            None
        }
    }
}
