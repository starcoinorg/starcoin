use crate::{BlockData, TransactionData};
use anyhow::Result;
use futures_util::compat::Future01CompatExt;
use jsonrpc_core_client::RpcError;
use starcoin_rpc_api::chain::ChainClient;
use starcoin_rpc_api::types::{
    BlockHeaderView, BlockTransactionsView, BlockView, ChainInfoView, TransactionEventView,
    TransactionInfoView, TransactionView,
};

pub struct BlockClient {
    node_client: ChainClient,
}

impl BlockClient {
    pub fn new(chain_client: ChainClient) -> Self {
        Self {
            node_client: chain_client,
        }
    }
    pub async fn get_block_whole_by_height(&self, height: u64) -> Result<BlockData, RpcError> {
        let block: BlockView = self
            .node_client
            .get_block_by_number(height)
            .compat()
            .await?;
        let mut txn_infos: Vec<TransactionInfoView> = self
            .node_client
            .get_block_txn_infos(block.header.block_hash)
            .compat()
            .await?;
        let mut txns_data = vec![];

        {
            let blockmeta_info = txn_infos.remove(0);
            let txn: TransactionView = self
                .node_client
                .get_transaction(blockmeta_info.transaction_hash)
                .compat()
                .await?;
            let events: Vec<TransactionEventView> = self
                .node_client
                .get_events_by_txn_hash(blockmeta_info.transaction_hash)
                .compat()
                .await?;
            txns_data.push(TransactionData {
                info: blockmeta_info,
                block_metadata: txn.block_metadata,
                user_transaction: txn.user_transaction,
                events,
            })
        }
        let user_transactions = match &block.body {
            BlockTransactionsView::Hashes(_) => unreachable!(),
            BlockTransactionsView::Full(txns) => txns.clone(),
        };
        for (txn_info, user_txn) in txn_infos.into_iter().zip(user_transactions) {
            let events: Vec<TransactionEventView> = self
                .node_client
                .get_events_by_txn_hash(txn_info.transaction_hash)
                .compat()
                .await?;
            txns_data.push(TransactionData {
                info: txn_info,
                events,
                user_transaction: Some(user_txn),
                block_metadata: None,
            })
        }
        Ok(BlockData { block, txns_data })
    }
    pub async fn get_chain_head(&self) -> Result<BlockHeaderView, RpcError> {
        let chain_info: ChainInfoView = self.node_client.info().compat().await?;
        Ok(chain_info.head)
    }
}
