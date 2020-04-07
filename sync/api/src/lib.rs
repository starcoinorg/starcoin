use anyhow::Result;
use parking_lot::RwLock;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_types::block::BlockNumber;
use std::sync::Arc;

#[derive(Clone)]
pub struct SyncMetadata(Arc<RwLock<SyncMetadataInner>>);

pub struct SyncMetadataInner {
    syncing: bool,
    pivot: Option<BlockNumber>,
}

impl SyncMetadata {
    pub fn new(config: Arc<NodeConfig>) -> SyncMetadata {
        info!("is_state_sync : {}", config.sync.is_state_sync());
        let inner = SyncMetadataInner {
            syncing: config.sync.is_state_sync(),
            pivot: None,
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
        Ok(())
    }

    pub fn is_state_sync(&self) -> Result<bool> {
        Ok(self.0.read().syncing)
    }

    pub fn get_pivot(&self) -> Result<Option<BlockNumber>> {
        Ok(self.0.read().pivot.clone())
    }
}
