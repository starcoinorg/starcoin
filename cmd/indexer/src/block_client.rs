use crate::{BlockData, TransactionData};
use anyhow::Result;
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
        let block: Option<BlockView> = self.node_client.get_block_by_number(height).await?;
        let block = block.ok_or_else(|| {
            RpcError::Other(Box::new(
                failure::err_msg(format!("cannot find block of height {}", height)).compat(),
            ))
        })?;
        let mut txn_infos: Vec<TransactionInfoView> = self
            .node_client
            .get_block_txn_infos(block.header.block_hash)
            .await?;
        let mut txns_data = vec![];

        {
            let txn_info = txn_infos.remove(0);
            let txn: Option<TransactionView> = self
                .node_client
                .get_transaction(txn_info.transaction_hash)
                .await?;
            let txn = txn.ok_or_else(|| {
                RpcError::Other(Box::new(
                    failure::err_msg(format!(
                        "cannot find txn with id {}",
                        txn_info.transaction_hash
                    ))
                    .compat(),
                ))
            })?;

            let events: Vec<TransactionEventView> = self
                .node_client
                .get_events_by_txn_hash(txn_info.transaction_hash)
                .await?;
            txns_data.push(TransactionData {
                info: txn_info,
                block_metadata: txn.block_metadata,
                user_transaction: txn.user_transaction,
                events,
                timestamp: block.header.timestamp.0,
            })
        }
        let user_transactions = match &block.body {
            BlockTransactionsView::Hashes(_) => unreachable!(),
            BlockTransactionsView::Full(txns) => txns.clone(),
        };
        let fetch_events_tasks = txn_infos
            .iter()
            .map(|txn_info| txn_info.transaction_hash)
            .map(|txn_hash| self.node_client.get_events_by_txn_hash(txn_hash));

        let events = futures_util::future::try_join_all(fetch_events_tasks).await?;

        for ((txn_info, events), user_txn) in
            txn_infos.into_iter().zip(events).zip(user_transactions)
        {
            txns_data.push(TransactionData {
                info: txn_info,
                events,
                user_transaction: Some(user_txn),
                block_metadata: None,
                timestamp: block.header.timestamp.0,
            })
        }
        Ok(BlockData { block, txns_data })
    }
    pub async fn get_chain_head(&self) -> Result<BlockHeaderView, RpcError> {
        let chain_info: ChainInfoView = self.node_client.info().await?;
        Ok(chain_info.head)
    }
}
