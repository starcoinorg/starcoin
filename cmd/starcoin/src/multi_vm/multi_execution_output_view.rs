use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::{TransactionEventResponse, TransactionInfoView};
use starcoin_vm2_types::view::{
    transaction_event_response::TransactionEventResponse as TransactionEventResponseV2,
    transaction_info_view::TransactionInfoView as TransactionInfoViewV2,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum MultiTransactionInfoView {
    VM1(TransactionInfoView),
    VM2(TransactionInfoViewV2),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum MultiTransactionEventResponse {
    VM1(TransactionEventResponse),
    VM2(TransactionEventResponseV2),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct MultiExecutionOutputView {
    pub txn_hash: HashValue,
    pub txn_info: Option<MultiTransactionInfoView>,
    pub events: Option<Vec<MultiTransactionEventResponse>>,
}
