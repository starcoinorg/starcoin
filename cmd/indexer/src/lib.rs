mod block_client;
mod es_sinker;
pub use block_client::BlockClient;
pub use es_sinker::{EsSinker, IndexConfig, LocalTipInfo};

use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::{
    BlockMetadataView, BlockView, SignedUserTransactionView, StrView, TransactionEventView,
    TransactionInfoView, TransactionVMStatus,
};
use starcoin_types::vm_error::AbortLocation;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionData {
    #[serde(flatten)]
    pub info: TransactionInfoEsView,
    pub block_metadata: Option<BlockMetadataView>,
    pub user_transaction: Option<SignedUserTransactionView>,
    pub events: Vec<TransactionEventView>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionInfoEsView {
    pub block_hash: HashValue,
    pub block_number: StrView<u64>,
    /// The hash of this transaction.
    pub transaction_hash: HashValue,
    pub transaction_index: u32,
    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root_hash: HashValue,

    /// The amount of gas used.
    pub gas_used: StrView<u64>,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    #[serde(flatten)]
    pub status: TransactionVMStatusEsView,
}

impl From<TransactionInfoView> for TransactionInfoEsView {
    fn from(info: TransactionInfoView) -> Self {
        Self {
            block_hash: info.block_hash,
            block_number: info.block_number,
            transaction_hash: info.transaction_hash,
            transaction_index: info.transaction_index,
            state_root_hash: info.state_root_hash,
            event_root_hash: info.event_root_hash,
            gas_used: info.gas_used,
            status: info.status.into(),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "status_content")]
#[allow(clippy::upper_case_acronyms)]
pub enum TransactionVMStatusEsView {
    Executed,
    OutOfGas,
    MoveAbort {
        location: AbortLocation,
        abort_code: StrView<u64>,
    },
    ExecutionFailure {
        location: AbortLocation,
        function: u16,
        code_offset: u16,
    },
    MiscellaneousError,
    Discard {
        status_code: StrView<u64>,
    },
}

impl From<TransactionVMStatus> for TransactionVMStatusEsView {
    fn from(s: TransactionVMStatus) -> Self {
        match s {
            TransactionVMStatus::Executed => Self::Executed,
            TransactionVMStatus::OutOfGas => Self::OutOfGas,
            TransactionVMStatus::MoveAbort {
                location,
                abort_code,
            } => Self::MoveAbort {
                location,
                abort_code,
            },
            TransactionVMStatus::ExecutionFailure {
                location,
                function,
                code_offset,
            } => Self::ExecutionFailure {
                location,
                function,
                code_offset,
            },
            TransactionVMStatus::MiscellaneousError => Self::MiscellaneousError,
            TransactionVMStatus::Discard { status_code } => Self::Discard { status_code },
        }
    }
}

#[derive(Clone, Debug)]
pub struct BlockData {
    pub block: BlockView,
    pub txns_data: Vec<TransactionData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BlockWithMetadata {
    #[serde(flatten)]
    block: BlockView,
    metadata: Option<BlockMetadataView>,
}

#[cfg(test)]
mod tests {
    use crate::{TransactionInfoEsView, TransactionVMStatusEsView};
    use starcoin_rpc_api::types::StrView;

    #[test]
    fn test_info_view() {
        let v = TransactionInfoEsView {
            block_hash: Default::default(),
            block_number: StrView(1),
            transaction_hash: Default::default(),
            transaction_index: 0,
            state_root_hash: Default::default(),
            event_root_hash: Default::default(),
            gas_used: StrView(0),
            status: TransactionVMStatusEsView::Executed,
        };

        let expected = r#"
        {"block_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","block_number":"1","transaction_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","transaction_index":0,"state_root_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","event_root_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","gas_used":"0","status":"Executed"}
        "#;
        assert_eq!(serde_json::to_string(&v).unwrap().as_str(), expected.trim());
        let v = TransactionInfoEsView {
            block_hash: Default::default(),
            block_number: StrView(1),
            transaction_hash: Default::default(),
            transaction_index: 0,
            state_root_hash: Default::default(),
            event_root_hash: Default::default(),
            gas_used: StrView(0),
            status: TransactionVMStatusEsView::Discard {
                status_code: StrView(1000),
            },
        };
        let expected = r#"
        {"block_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","block_number":"1","transaction_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","transaction_index":0,"state_root_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","event_root_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","gas_used":"0","status":"Discard","status_content":{"status_code":"1000"}}
        "#;
        assert_eq!(serde_json::to_string(&v).unwrap().as_str(), expected.trim());
    }
}
