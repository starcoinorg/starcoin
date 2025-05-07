use schemars::JsonSchema;
use crate::multi_vm::multi_execution_output_view::{
    MultiExecutionOutputView, MultiTransactionInfoView,
};
use serde::{Deserialize, Serialize};
use starcoin_rpc_api::multi_dry_run_output_view::MultiDryRunOutputView;
use starcoin_rpc_api::muti_raw_user_transaction_view::MultiRawUserTransactionView;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct MultiExecuteResultView {
    pub raw_txn: MultiRawUserTransactionView,
    pub raw_txn_hex: String,
    pub dry_run_output: MultiDryRunOutputView,
    pub execute_output: Option<MultiExecutionOutputView>,
}

impl MultiExecuteResultView {
    pub fn new(
        raw_txn: MultiRawUserTransactionView,
        raw_txn_hex: String,
        dry_run_output: MultiDryRunOutputView,
    ) -> Self {
        Self {
            raw_txn,
            raw_txn_hex,
            dry_run_output,
            execute_output: None,
        }
    }
    pub fn get_transaction_info(&self) -> Option<MultiTransactionInfoView> {
        self.execute_output
            .clone()
            .map(|output| output.txn_info)
            .unwrap()
    }
}
