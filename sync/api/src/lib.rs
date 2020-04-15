pub mod sync_messages;

use anyhow::Result;
use dyn_clone::{clone_box, DynClone};
use parking_lot::RwLock;
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::block::BlockNumber;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait StateSyncReset: DynClone + Send + Sync {
    async fn reset(&self, root: HashValue);
}

#[derive(Clone)]
pub struct SyncMetadata(Arc<RwLock<SyncMetadataInner>>);

pub struct SyncMetadataInner {
    is_state_sync: bool,
    pivot: Option<BlockNumber>,
    state_sync_address: Option<Box<dyn StateSyncReset>>,
    state_sync_done: bool,
    block_sync_done: bool,
}

impl SyncMetadata {
    pub fn new(config: Arc<NodeConfig>) -> SyncMetadata {
        info!("is_state_sync : {}", config.sync.is_state_sync());
        let inner = SyncMetadataInner {
            is_state_sync: config.sync.is_state_sync(),
            pivot: None,
            state_sync_address: None,
            state_sync_done: false,
            block_sync_done: false,
        };
        SyncMetadata(Arc::new(RwLock::new(inner)))
    }

    pub fn can_sync_state(&self) -> bool {
        self.is_state_sync() && !self.0.read().state_sync_done && !self.0.read().block_sync_done
    }
    pub fn update_pivot(&self, pivot: BlockNumber) -> Result<()> {
        assert!(self.can_sync_state(), "cat not update pivot.");
        self.0.write().pivot = Some(pivot);
        Ok(())
    }

    pub fn state_sync_done(&self) -> Result<()> {
        assert!(self.is_state_sync(), "chain is not in fast sync mode.");
        assert!(!self.0.read().state_sync_done, "state sync already done.");
        let mut lock = self.0.write();
        lock.state_sync_done = true;
        self.both_done();
        Ok(())
    }

    pub fn block_sync_done(&self) -> Result<()> {
        assert!(self.is_state_sync(), "chain is not in fast sync mode.");
        assert!(!self.0.read().block_sync_done, "block sync already done.");
        let mut lock = self.0.write();
        lock.state_sync_done = true;
        self.both_done();
        Ok(())
    }

    fn both_done(&self) -> Result<()> {
        if self.0.read().state_sync_done && self.0.read().block_sync_done {
            let mut lock = self.0.write();
            lock.pivot = None;
            lock.state_sync_address = None;
            //todo: send done event
        }
        Ok(())
    }

    fn is_state_sync(&self) -> bool {
        self.0.read().is_state_sync
    }

    pub fn get_pivot(&self) -> Result<Option<BlockNumber>> {
        Ok(self.0.read().pivot.clone())
    }

    pub fn update_address(&self, address: &(dyn StateSyncReset + 'static)) -> Result<()> {
        assert!(self.can_sync_state(), "cat not update address.");
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
