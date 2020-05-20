pub mod sync_messages;

use actix::Addr;
use anyhow::Result;
use dyn_clone::{clone_box, DynClone};
use parking_lot::RwLock;
use starcoin_bus::{Broadcast, BusActor};
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::block::BlockNumber;
use starcoin_types::system_events::SyncDone;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait StateSyncReset: DynClone + Send + Sync {
    async fn reset(
        &self,
        state_root: HashValue,
        txn_accumulator_root: HashValue,
        block_accumulator_root: HashValue,
    );
    async fn act(&self);
}

#[derive(Clone)]
pub struct SyncMetadata(Arc<RwLock<SyncMetadataInner>>);

pub struct SyncMetadataInner {
    is_state_sync: bool,
    pivot_behind: Option<(BlockNumber, u64)>,
    pivot_connected: bool,
    state_sync_address: Option<Box<dyn StateSyncReset>>,
    state_sync_done: bool,
    block_sync_done: bool,
    bus: Addr<BusActor>,
    state_sync_failed: Option<bool>,
}

impl SyncMetadata {
    pub fn new(config: Arc<NodeConfig>, bus: Addr<BusActor>) -> SyncMetadata {
        info!("is_state_sync : {}", config.sync.is_state_sync());
        let inner = SyncMetadataInner {
            is_state_sync: config.sync.is_state_sync(),
            pivot_behind: None,
            pivot_connected: false,
            state_sync_address: None,
            state_sync_done: false,
            block_sync_done: false,
            bus,
            state_sync_failed: Some(false),
        };
        SyncMetadata(Arc::new(RwLock::new(inner)))
    }

    pub fn state_syncing(&self) -> bool {
        self.fast_sync_mode() && (!self.0.read().state_sync_done || !self.0.read().block_sync_done)
    }

    pub fn update_pivot(&self, pivot: BlockNumber, behind: u64) -> Result<()> {
        assert!(self.state_syncing(), "cat not update pivot.");
        assert!(pivot > 0, "pivot must be positive integer.");
        assert!(behind > 0, "behind must be positive integer.");
        self.0.write().pivot_behind = Some((pivot, behind));
        self.0.write().pivot_connected = false;
        Ok(())
    }

    pub fn update_failed(&self, failed: bool) {
        if !self.is_sync_done() {
            self.0.write().state_sync_failed = Some(failed);
        }
    }

    pub fn is_failed(&self) -> bool {
        if let Some(failed) = self.0.read().state_sync_failed {
            return failed;
        }
        false
    }

    pub fn state_sync_done(&self) -> Result<()> {
        assert!(self.fast_sync_mode(), "chain is not in fast sync mode.");
        assert!(!self.0.read().state_sync_done, "state sync already done.");
        let mut lock = self.0.write();
        lock.state_sync_done = true;
        drop(lock);
        self.sync_done()?;
        info!("state sync done.");
        Ok(())
    }

    pub fn pivot_connected_succ(&self) -> Result<()> {
        let mut lock = self.0.write();
        lock.pivot_connected = true;
        info!("pivot block connected done.");
        Ok(())
    }

    pub fn pivot_connected(&self) -> bool {
        self.0.read().pivot_connected
    }

    pub fn block_sync_done(&self) -> Result<()> {
        info!("do block_sync_done");
        let read_lock = self.0.read();
        if !read_lock.block_sync_done && (read_lock.pivot_connected || !self.fast_sync_mode()) {
            drop(read_lock);
            let mut lock = self.0.write();
            lock.block_sync_done = true;
            drop(lock);
            self.sync_done()?;
            info!("block sync done.");
        }
        Ok(())
    }

    pub fn is_sync_done(&self) -> bool {
        (self.fast_sync_mode() && self.0.read().state_sync_done && self.0.read().block_sync_done)
            || (!self.fast_sync_mode() && self.0.read().block_sync_done)
    }

    fn sync_done(&self) -> Result<()> {
        if self.is_sync_done() {
            let mut lock = self.0.write();
            lock.pivot_behind = None;
            lock.state_sync_address = None;
            lock.state_sync_failed = None;
            lock.bus.do_send(Broadcast { msg: SyncDone });
            info!("state sync and block sync done.");
        }
        Ok(())
    }

    pub fn state_done(&self) -> bool {
        self.0.read().state_sync_done
    }

    pub fn fast_sync_mode(&self) -> bool {
        self.0.read().is_state_sync
    }

    pub fn get_pivot(&self) -> Result<Option<BlockNumber>> {
        Ok(match self.0.read().pivot_behind {
            None => None,
            Some((pivot, _behind)) => Some(pivot),
        })
    }

    pub fn get_latest(&self) -> Option<BlockNumber> {
        match self.0.read().pivot_behind {
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
