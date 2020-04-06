use anyhow::Result;
use types::block::BlockNumber;

// pub enum SyncMetadata {
//     Bool(bool),
//     OptionNumber(Option<BlockNumber>),
//     None,
// }
//
// #[async_trait::async_trait(? Send)]
// pub trait SyncMetadataAsyncService: Clone + std::marker::Unpin {
//     async fn update_pivot(&self, pivot: BlockNumber) -> Result<()>;
//
//     async fn sync_done(&self) -> Result<()>;
//
//     async fn is_state_sync(&self) -> Result<bool>;
//
//     async fn get_pivot(&self) -> Result<Option<BlockNumber>>;
// }

// pub trait SyncMetadataService: std::marker::Send + std::marker::Sync + std::marker::Unpin + Clone {
//     fn update_pivot(&self, pivot: BlockNumber) -> Result<()>;
//
//     fn sync_done(&self) -> Result<()>;
//
//     fn is_state_sync(&self) -> Result<bool>;
//
//     fn get_pivot(&self) -> Result<Option<BlockNumber>>;
// }
