// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod contract_call;
mod node_api_types;
pub mod pubsub;
pub use contract_call::*;
use jsonrpc_core_client::RpcChannel;
pub use node_api_types::*;
use scs::SCSCodec;
use serde::Deserialize;
use serde::Serialize;
use serde_helpers::{deserialize_binary, deserialize_u64, serialize_binary, serialize_u64};
use starcoin_crypto::HashValue;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::{
    Block, BlockBody, BlockHeader, BlockNumber, BlockSummary, EpochUncleSummary, UncleSummary,
};
use starcoin_types::contract_event::{ContractEvent, ContractEventInfo};
use starcoin_types::event::EventKey;
use starcoin_types::genesis_config;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::peer_info::{PeerId, PeerInfo};
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::authenticator::{AuthenticationKey, TransactionAuthenticator};
use starcoin_types::transaction::RawUserTransaction;
use starcoin_types::vm_error::AbortLocation;
use starcoin_types::U256;
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::transaction::{SignedUserTransaction, Transaction, TransactionInfo};
use starcoin_vm_types::vm_status::KeptVMStatus;
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockHeaderView {
    pub block_hash: HashValue,
    /// Parent hash.
    pub parent_hash: HashValue,
    /// Block timestamp.
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub timestamp: u64,
    /// Block number.
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub number: BlockNumber,
    /// Block author.
    pub author: AccountAddress,
    /// Block author auth key.
    pub author_auth_key: Option<AuthenticationKey>,
    /// The transaction accumulator root hash after executing this block.
    pub accumulator_root: HashValue,
    /// The parent block accumulator root hash.
    pub parent_block_accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    pub state_root: HashValue,
    /// Gas used for contracts execution.
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub gas_used: u64,
    /// Block difficulty
    pub difficulty: U256,
    /// Consensus nonce field.
    pub nonce: u32,
    /// hash for block body
    pub body_hash: HashValue,
    /// The chain id
    pub chain_id: u8,
}
impl From<BlockHeader> for BlockHeaderView {
    fn from(origin: BlockHeader) -> Self {
        BlockHeaderView {
            block_hash: origin.id(),
            parent_hash: origin.parent_hash,
            timestamp: origin.timestamp,
            number: origin.number,
            author: origin.author,
            author_auth_key: origin.author_auth_key,
            accumulator_root: origin.accumulator_root,
            parent_block_accumulator_root: origin.parent_block_accumulator_root,
            state_root: origin.state_root,
            gas_used: origin.gas_used,
            difficulty: origin.difficulty,
            nonce: origin.nonce,
            body_hash: origin.body_hash,
            chain_id: origin.chain_id.id(),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RawUserTransactionView {
    /// Sender's address.
    pub sender: AccountAddress,
    // Sequence number of this transaction corresponding to sender's account.
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub sequence_number: u64,

    // The transaction payload in scs bytes.
    #[serde(
        serialize_with = "serialize_binary",
        deserialize_with = "deserialize_binary"
    )]
    pub payload: Vec<u8>,

    // Maximal total gas specified by wallet to spend for this transaction.
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub max_gas_amount: u64,
    // Maximal price can be paid per gas.
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub gas_unit_price: u64,
    // The token code for pay transaction gas, Default is STC token code.
    pub gas_token_code: String,
    // Expiration timestamp for this transaction. timestamp is represented
    // as u64 in seconds from Unix Epoch. If storage is queried and
    // the time returned is greater than or equal to this time and this
    // transaction has not been included, you can be certain that it will
    // never be included.
    // A transaction that doesn't expire is represented by a very large value like
    // u64::max_value().
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub expiration_timestamp_secs: u64,
    pub chain_id: u8,
}

impl TryFrom<RawUserTransaction> for RawUserTransactionView {
    type Error = anyhow::Error;

    fn try_from(origin: RawUserTransaction) -> Result<Self, Self::Error> {
        Ok(RawUserTransactionView {
            sender: origin.sender(),
            sequence_number: origin.sequence_number(),
            max_gas_amount: origin.max_gas_amount(),
            gas_unit_price: origin.gas_unit_price(),
            gas_token_code: origin.gas_token_code(),
            expiration_timestamp_secs: origin.expiration_timestamp_secs(),
            chain_id: origin.chain_id().id(),
            payload: origin.into_payload().encode()?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SignedUserTransactionView {
    pub transaction_hash: HashValue,
    /// The raw transaction
    pub raw_txn: RawUserTransactionView,

    /// Public key and signature to authenticate
    pub authenticator: TransactionAuthenticator,
}

impl TryFrom<SignedUserTransaction> for SignedUserTransactionView {
    type Error = anyhow::Error;

    fn try_from(txn: SignedUserTransaction) -> Result<Self, Self::Error> {
        let auth = txn.authenticator();
        let txn_hash = txn.id();
        Ok(SignedUserTransactionView {
            transaction_hash: txn_hash,
            raw_txn: txn.into_raw_transaction().try_into()?,
            authenticator: auth,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BlockMetadataView {
    /// Parent block hash.
    pub parent_hash: HashValue,
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub timestamp: u64,
    pub author: AccountAddress,
    pub author_auth_key: Option<AuthenticationKey>,
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub uncles: u64,
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub number: BlockNumber,
    pub chain_id: u8,
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub parent_gas_used: u64,
}

impl From<BlockMetadata> for BlockMetadataView {
    fn from(origin: BlockMetadata) -> Self {
        let (
            parent_hash,
            timestamp,
            author,
            author_auth_key,
            uncles,
            number,
            chain_id,
            parent_gas_used,
        ) = origin.into_inner();
        BlockMetadataView {
            parent_hash,
            timestamp,
            author,
            author_auth_key,
            uncles,
            number,
            chain_id: chain_id.id(),
            parent_gas_used,
        }
    }
}
impl Into<BlockMetadata> for BlockMetadataView {
    fn into(self) -> BlockMetadata {
        let BlockMetadataView {
            parent_hash,
            timestamp,
            author,
            author_auth_key,
            uncles,
            number,
            chain_id,
            parent_gas_used,
        } = self;
        BlockMetadata::new(
            parent_hash,
            timestamp,
            author,
            author_auth_key,
            uncles,
            number,
            genesis_config::ChainId::new(chain_id),
            parent_gas_used,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TransactionView {
    pub block_hash: HashValue,
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub block_number: BlockNumber,
    pub transaction_hash: HashValue,
    pub transaction_index: u32,
    pub block_metadata: Option<BlockMetadataView>,
    pub user_transaction: Option<SignedUserTransactionView>,
}

impl TransactionView {
    pub fn new(txn: Transaction, block: &Block) -> anyhow::Result<Self> {
        let transaction_hash = txn.id();
        let block_hash = block.id();
        let block_number = block.header.number;
        let transaction_index = match &txn {
            Transaction::BlockMetadata(_) => 0,
            _ => block
                .transactions()
                .iter()
                .position(|t| t.id() == transaction_hash)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "cannot find txn {} in block {}",
                        transaction_hash,
                        block_hash
                    )
                })? as u32,
        };

        let (meta, txn) = match txn {
            Transaction::BlockMetadata(meta) => (Some(meta.into()), None),
            Transaction::UserTransaction(t) => (None, Some(t.try_into()?)),
        };
        Ok(Self {
            block_hash,
            block_number,
            transaction_hash,
            transaction_index: transaction_index + 1,
            block_metadata: meta,
            user_transaction: txn,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BlockTransactionsView {
    Hashes(Vec<HashValue>),
    Full(Vec<SignedUserTransactionView>),
}

impl BlockTransactionsView {
    pub fn txn_hashes(&self) -> Vec<HashValue> {
        match self {
            Self::Hashes(h) => h.clone(),
            Self::Full(f) => f.iter().map(|t| t.transaction_hash).collect(),
        }
    }
}

impl TryFrom<Vec<SignedUserTransaction>> for BlockTransactionsView {
    type Error = anyhow::Error;

    fn try_from(txns: Vec<SignedUserTransaction>) -> Result<Self, Self::Error> {
        Ok(BlockTransactionsView::Full(
            txns.into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl From<Vec<HashValue>> for BlockTransactionsView {
    fn from(txns: Vec<HashValue>) -> Self {
        BlockTransactionsView::Hashes(txns)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockView {
    pub header: BlockHeaderView,
    pub body: BlockTransactionsView,
    pub uncles: Vec<BlockHeaderView>,
}

impl TryFrom<Block> for BlockView {
    type Error = anyhow::Error;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        let (header, body) = block.into_inner();
        let BlockBody {
            transactions,
            uncles,
        } = body;
        Ok(BlockView {
            header: header.into(),
            uncles: uncles
                .unwrap_or_default()
                .into_iter()
                .map(|h| h.into())
                .collect(),
            body: transactions.try_into()?,
        })
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockSummaryView {
    pub header: BlockHeaderView,
    pub uncles: Vec<BlockHeaderView>,
}
impl From<BlockSummary> for BlockSummaryView {
    fn from(summary: BlockSummary) -> Self {
        BlockSummaryView {
            header: summary.block_header.into(),
            uncles: summary
                .uncles
                .into_iter()
                .map(|uncle| uncle.into())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionInfoView {
    pub block_hash: HashValue,
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub block_number: BlockNumber,
    /// The hash of this transaction.
    pub transaction_hash: HashValue,
    pub transaction_index: u32,
    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root_hash: HashValue,

    /// The amount of gas used.
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    pub status: TransactionVMStatus,
}

impl TransactionInfoView {
    pub fn new(txn_info: TransactionInfo, txn_block: &Block) -> anyhow::Result<Self> {
        let block_hash = txn_block.id();
        let transaction_hash = txn_info.transaction_hash();

        // if not found in block, it means it's block meta txn.
        let index = txn_block
            .transactions()
            .iter()
            .position(|t| t.id() == transaction_hash);

        Ok(TransactionInfoView {
            block_hash,
            block_number: txn_block.header().number,
            transaction_hash,
            transaction_index: index.map(|i| i + 1).unwrap_or_default() as u32,
            state_root_hash: txn_info.state_root_hash(),
            event_root_hash: txn_info.event_root_hash(),
            gas_used: txn_info.gas_used(),
            status: TransactionVMStatus::from(txn_info.status().clone()),
        })
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionVMStatus {
    Executed,
    OutOfGas,
    MoveAbort {
        location: AbortLocation,
        #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
        abort_code: u64,
    },
    ExecutionFailure {
        location: AbortLocation,
        function: u16,
        code_offset: u16,
    },
    MiscellaneousError,
}

impl From<KeptVMStatus> for TransactionVMStatus {
    fn from(origin: KeptVMStatus) -> Self {
        match origin {
            KeptVMStatus::Executed => TransactionVMStatus::Executed,
            KeptVMStatus::OutOfGas => TransactionVMStatus::OutOfGas,
            KeptVMStatus::MoveAbort(l, c) => TransactionVMStatus::MoveAbort {
                location: l,
                abort_code: c,
            },
            KeptVMStatus::ExecutionFailure {
                location,
                function,
                code_offset,
            } => TransactionVMStatus::ExecutionFailure {
                location,
                function,
                code_offset,
            },
            KeptVMStatus::MiscellaneousError => TransactionVMStatus::MiscellaneousError,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct TransactionEventView {
    pub block_hash: Option<HashValue>,
    pub block_number: Option<BlockNumber>,
    pub transaction_hash: Option<HashValue>,
    // txn index in block
    pub transaction_index: Option<u32>,

    #[serde(
        serialize_with = "serialize_binary",
        deserialize_with = "deserialize_binary"
    )]
    pub data: Vec<u8>,
    pub type_tag: TypeTag,
    pub event_key: EventKey,
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub event_seq_number: u64,
}

impl From<ContractEventInfo> for TransactionEventView {
    fn from(info: ContractEventInfo) -> Self {
        TransactionEventView {
            block_hash: Some(info.block_hash),
            block_number: Some(info.block_number),
            transaction_hash: Some(info.transaction_hash),
            transaction_index: Some(info.transaction_index),
            data: info.event.event_data().to_vec(),
            type_tag: info.event.type_tag().clone(),
            event_key: *info.event.key(),
            event_seq_number: info.event.sequence_number(),
        }
    }
}

impl TransactionEventView {
    pub fn new(
        block_hash: Option<HashValue>,
        block_number: Option<BlockNumber>,
        transaction_hash: Option<HashValue>,
        transaction_index: Option<u32>,
        contract_event: &ContractEvent,
    ) -> Self {
        Self {
            block_hash,
            block_number,
            transaction_hash,
            transaction_index,
            data: contract_event.event_data().to_vec(),
            type_tag: contract_event.type_tag().clone(),
            event_key: *contract_event.key(),
            event_seq_number: contract_event.sequence_number(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UncleSummaryView {
    /// total uncle
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub uncles: u64,
    /// sum(number of the block which contain uncle block - uncle parent block number).
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub sum: u64,
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub avg: u64,
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub time_sum: u64,
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub time_avg: u64,
}

impl From<UncleSummary> for UncleSummaryView {
    fn from(origin: UncleSummary) -> Self {
        Self {
            uncles: origin.uncles,
            sum: origin.sum,
            avg: origin.avg,
            time_sum: origin.time_sum,
            time_avg: origin.time_avg,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpochUncleSummaryView {
    /// epoch number
    #[serde(deserialize_with = "deserialize_u64", serialize_with = "serialize_u64")]
    pub epoch: u64,
    pub number_summary: UncleSummaryView,
    pub epoch_summary: UncleSummaryView,
}

impl From<EpochUncleSummary> for EpochUncleSummaryView {
    fn from(origin: EpochUncleSummary) -> Self {
        EpochUncleSummaryView {
            epoch: origin.epoch,
            number_summary: origin.number_summary.into(),
            epoch_summary: origin.epoch_summary.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainInfoView {
    pub chain_id: u8,
    pub genesis_hash: HashValue,
    pub head: BlockHeaderView,
    pub total_difficulty: U256,
}

impl From<ChainInfo> for ChainInfoView {
    fn from(info: ChainInfo) -> Self {
        let (chain_id, genesis_hash, status) = info.into_inner();
        let (head, total_difficulty) = status.into_inner();
        Self {
            chain_id: chain_id.into(),
            genesis_hash,
            head: head.into(),
            total_difficulty,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerInfoView {
    pub peer_id: PeerId,
    pub chain_info: ChainInfoView,
}

impl From<PeerInfo> for PeerInfoView {
    fn from(info: PeerInfo) -> Self {
        let (peer_id, chain_info) = info.into_inner();
        Self {
            peer_id,
            chain_info: chain_info.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectLocal;

impl ServiceRequest for ConnectLocal {
    type Response = RpcChannel;
}
