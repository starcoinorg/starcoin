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
    syncing: bool,
    pivot: Option<BlockNumber>,
    state_sync_address: Option<Box<dyn StateSyncReset>>,
}

impl SyncMetadata {
    pub fn new(config: Arc<NodeConfig>) -> SyncMetadata {
        info!("is_state_sync : {}", config.sync.is_state_sync());
        let inner = SyncMetadataInner {
            syncing: config.sync.is_state_sync(),
            pivot: None,
            state_sync_address: None,
        };
        SyncMetadata(Arc::new(RwLock::new(inner)))
    }

    pub fn update_pivot(&self, pivot: BlockNumber) -> Result<()> {
        assert!(self.0.read().syncing, "chain is not in fast sync mode.");
        self.0.write().pivot = Some(pivot);
        Ok(())
    }

    pub fn sync_done(&self) -> Result<()> {
        let mut lock = self.0.write();
        lock.syncing = false;
        lock.pivot = None;
        lock.state_sync_address = None;
        Ok(())
    }

    pub fn is_state_sync(&self) -> Result<bool> {
        Ok(self.0.read().syncing)
    }

    pub fn get_pivot(&self) -> Result<Option<BlockNumber>> {
        Ok(self.0.read().pivot.clone())
    }

    pub fn update_address(&self, address: &(dyn StateSyncReset + 'static)) -> Result<()> {
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
