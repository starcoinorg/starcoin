use crate::helper::do_response_get_txns;
use anyhow::Result;
use futures::channel::mpsc;
use starcoin_sync_api::sync_messages::{GetTxns, TransactionsData};
use starcoin_txpool_api::TxPoolSyncService;
use txpool::TxPoolService;
#[derive(Clone)]
pub struct GetTxnsHandler {
    pool: TxPoolService,
}

impl GetTxnsHandler {
    pub fn new(txpool: TxPoolService) -> GetTxnsHandler {
        Self { pool: txpool }
    }
}
// TODO: we can do more logic here
impl GetTxnsHandler {
    pub async fn handle(self, responder: mpsc::Sender<Vec<u8>>, _msg: GetTxns) -> Result<()> {
        let data = self.pool.get_pending_txns(None);
        do_response_get_txns(responder, TransactionsData { txns: data }).await
    }
}
