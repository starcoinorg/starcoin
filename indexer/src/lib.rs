mod block_client;
mod es_sinker;
pub use block_client::BlockClient;
pub use es_sinker::{EsSinker, IndexConfig};

use serde::{Deserialize, Serialize};
use starcoin_rpc_api::types::{
    BlockMetadataView, BlockView, SignedUserTransactionView, TransactionEventView,
    TransactionInfoView,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionData {
    #[serde(flatten)]
    pub info: TransactionInfoView,
    pub block_metadata: Option<BlockMetadataView>,
    pub user_transaction: Option<SignedUserTransactionView>,
    pub events: Vec<TransactionEventView>,
}
#[derive(Clone, Debug)]
pub struct BlockData {
    pub block: BlockView,
    pub txns_data: Vec<TransactionData>,
}
