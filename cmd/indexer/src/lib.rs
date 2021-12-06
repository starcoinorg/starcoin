mod block_client;
mod es_sinker;
pub use block_client::BlockClient;
pub use es_sinker::{EsSinker, IndexConfig, LocalTipInfo};

use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::{
    BlockHeaderView, BlockMetadataView, BlockView, SignedUserTransactionView, StrView,
    TransactionEventView, TransactionInfoView, TransactionStatusView, TypeTagView,
};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::BlockNumber;
use starcoin_types::event::EventKey;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::vm_error::AbortLocation;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionData {
    #[serde(flatten)]
    pub info: TransactionInfoEsView,
    pub block_metadata: Option<BlockMetadataView>,
    pub user_transaction: Option<SignedUserTransactionView>,
    pub events: Vec<TransactionEventEsView>,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct TransactionEventEsView {
    pub block_hash: Option<HashValue>,
    pub block_number: Option<StrView<BlockNumber>>,
    pub transaction_hash: Option<HashValue>,
    // txn index in block
    pub transaction_index: Option<u32>,
    pub transaction_global_index: Option<StrView<u64>>,
    pub data: StrView<Vec<u8>>,
    pub type_tag: TypeTagView,
    pub event_index: Option<u32>,
    pub event_key: EventKey,
    pub event_seq_number: StrView<u64>,
}
impl From<TransactionEventView> for TransactionEventEsView {
    fn from(event: TransactionEventView) -> Self {
        Self {
            block_hash: event.block_hash,
            block_number: event.block_number,
            transaction_hash: event.transaction_hash,
            transaction_index: event.transaction_index,
            transaction_global_index: event.transaction_global_index,
            data: event.data,
            type_tag: event.type_tag,
            event_index: event.event_index,
            event_key: event.event_key,
            event_seq_number: StrView(event.event_seq_number.0),
        }
    }
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

impl From<TransactionStatusView> for TransactionVMStatusEsView {
    fn from(s: TransactionStatusView) -> Self {
        match s {
            TransactionStatusView::Executed => Self::Executed,
            TransactionStatusView::OutOfGas => Self::OutOfGas,
            TransactionStatusView::MoveAbort {
                location,
                abort_code,
            } => Self::MoveAbort {
                location,
                abort_code,
            },
            TransactionStatusView::ExecutionFailure {
                location,
                function,
                code_offset,
            } => Self::ExecutionFailure {
                location,
                function,
                code_offset,
            },
            TransactionStatusView::MiscellaneousError => Self::MiscellaneousError,
            TransactionStatusView::Discard {
                status_code,
                status_code_name: _,
            } => Self::Discard { status_code },
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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BlockSimplified {
    header: BlockHeaderView,
    pub uncle_block_number: StrView<BlockNumber>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EventData {
    pub event_seq_number: StrView<u64>,
    pub block_hash: Option<HashValue>,
    pub block_number: Option<StrView<BlockNumber>>,
    pub transaction_hash: Option<HashValue>,
    // txn index in block
    pub transaction_index: Option<u32>,
    pub data: StrView<Vec<u8>>,
    pub type_tag: TypeTagView,
    pub event_key: EventKey,
    pub event_address: AccountAddress,
    pub tag_address: Option<AccountAddress>,
    pub tag_module: Option<String>,
    pub tag_name: Option<String>,
    pub timestamp: u64,
}

impl From<TransactionEventEsView> for EventData {
    fn from(event: TransactionEventEsView) -> Self {
        match event.clone().type_tag.0 {
            TypeTag::Struct(struct_tag) => Self {
                event_seq_number: event.event_seq_number,
                block_hash: event.block_hash,
                block_number: event.block_number,
                transaction_hash: event.transaction_hash,
                transaction_index: event.transaction_index,
                data: event.data,
                type_tag: event.type_tag,
                event_key: event.event_key,
                event_address: event.event_key.get_creator_address(),
                tag_address: Some(struct_tag.address),
                tag_module: Some(struct_tag.module.to_string()),
                tag_name: Some(struct_tag.name.to_string()),
                timestamp: 0, //set later from txn
            },
            _ => Self {
                event_seq_number: event.event_seq_number,
                block_hash: event.block_hash,
                block_number: event.block_number,
                transaction_hash: event.transaction_hash,
                transaction_index: event.transaction_index,
                data: event.data,
                type_tag: event.type_tag,
                event_key: event.event_key,
                event_address: event.event_key.get_creator_address(),
                tag_address: None,
                tag_module: None,
                tag_name: None,
                timestamp: 0, //set later from txn
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{TransactionEventEsView, TransactionInfoEsView, TransactionVMStatusEsView};
    use starcoin_crypto::HashValue;
    use starcoin_rpc_api::types::{StrView, TransactionEventView};
    use starcoin_types::account_address::AccountAddress;
    use starcoin_types::event::EventKey;
    use starcoin_types::language_storage::TypeTag;

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
    #[test]
    fn test_event() {
        let v = TransactionEventView {
            block_hash: Some(HashValue::zero()),
            block_number: Some(StrView(1)),
            transaction_hash: Some(HashValue::zero()),
            transaction_index: Some(0),
            transaction_global_index: Some(StrView(1)),
            data: StrView(vec![0]),
            type_tag: StrView(TypeTag::Bool),
            event_index: Some(0),
            event_key: EventKey::new_from_address(&AccountAddress::ZERO, 0),
            event_seq_number: StrView(0),
        };
        let event_view = TransactionEventEsView::from(v);
        let expected = r#"
        {"block_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","block_number":"1","transaction_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","transaction_index":0,"transaction_global_index":"1","data":"0x00","type_tag":"bool","event_index":0,"event_key":"0x000000000000000000000000000000000000000000000000","event_seq_number":"0"}
        "#;
        assert_eq!(
            serde_json::to_string(&event_view).unwrap().as_str(),
            expected.trim()
        );
    }
}
