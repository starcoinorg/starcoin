// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod node_api_types;
pub mod pubsub;

pub use node_api_types::*;
pub use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};

use jsonrpc_core_client::RpcChannel;
use scs::SCSCodec;
use serde::de::Error;
use serde::{Deserialize, Serializer};
use serde::{Deserializer, Serialize};
use serde_helpers::{
    deserialize_binary, deserialize_from_string, deserialize_from_string_opt, serialize_binary,
    serialize_to_string, serialize_to_string_opt,
};
use starcoin_crypto::{CryptoMaterialError, HashValue, ValidCryptoMaterialStringExt};
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
use starcoin_types::transaction::{RawUserTransaction, TransactionArgument};
use starcoin_types::vm_error::AbortLocation;
use starcoin_types::U256;
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{ModuleId, StructTag};
use starcoin_vm_types::parser::{parse_transaction_argument, parse_type_tag};
use starcoin_vm_types::transaction::authenticator::AccountPublicKey;
use starcoin_vm_types::transaction::{
    Script, SignedUserTransaction, Transaction, TransactionInfo, TransactionPayload,
};
use starcoin_vm_types::vm_status::KeptVMStatus;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

pub type ByteCode = Vec<u8>;
#[derive(Default, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TransactionRequest {
    /// Sender's address.
    pub sender: Option<AccountAddress>,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: Option<u64>,
    /// The transaction script to execute.
    #[serde(default)]
    pub script: Option<ScriptData>,
    /// module codes.
    #[serde(default)]
    pub modules: Vec<StrView<ByteCode>>,
    // Maximal total gas specified by wallet to spend for this transaction.
    pub max_gas_amount: Option<u64>,
    // Maximal price can be paid per gas.
    pub gas_unit_price: Option<u64>,
    // The token code for pay transaction gas, Default is STC token code.
    pub gas_token_code: Option<String>,
    // Expiration timestamp for this transaction. timestamp is represented
    // as u64 in seconds from Unix Epoch. If storage is queried and
    // the time returned is greater than or equal to this time and this
    // transaction has not been included, you can be certain that it will
    // never be included.
    // A transaction that doesn't expire is represented by a very large value like
    // u64::max_value().
    pub expiration_timestamp_secs: Option<u64>,
    pub chain_id: Option<u8>,
}

impl From<RawUserTransaction> for TransactionRequest {
    fn from(raw: RawUserTransaction) -> Self {
        let mut request = TransactionRequest {
            sender: Some(raw.sender()),
            sequence_number: Some(raw.sequence_number()),
            script: None,
            modules: vec![],
            max_gas_amount: Some(raw.max_gas_amount()),
            gas_unit_price: Some(raw.gas_unit_price()),
            gas_token_code: Some(raw.gas_token_code()),
            expiration_timestamp_secs: Some(raw.expiration_timestamp_secs()),
            chain_id: Some(raw.chain_id().id()),
        };
        match raw.into_payload() {
            TransactionPayload::Script(s) => {
                request.script = Some(s.into());
            }
            TransactionPayload::Package(p) => {
                let (_, m, s) = p.into_inner();
                request.script = s.map(Into::into);
                request.modules = m.into_iter().map(|m| StrView(m.into())).collect();
            }
        }
        request
    }
}

#[derive(Default, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DryRunTransactionRequest {
    #[serde(flatten)]
    pub transaction: TransactionRequest,
    /// Sender's public key
    pub sender_public_key: Option<StrView<AccountPublicKey>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ScriptData {
    pub code: StrView<ByteCodeOrScriptName>,
    #[serde(default)]
    pub type_args: Vec<TypeTagView>,
    #[serde(default)]
    pub args: Vec<TransactionArgumentView>,
}

impl From<Script> for ScriptData {
    fn from(s: Script) -> Self {
        let (code, ty_args, args) = s.into_inner();
        ScriptData {
            code: StrView(ByteCodeOrScriptName::ByteCode(code)),
            type_args: ty_args.into_iter().map(TypeTagView::from).collect(),
            args: args
                .into_iter()
                .map(TransactionArgumentView::from)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum ByteCodeOrScriptName {
    ByteCode(ByteCode),
    ScriptName(String),
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockHeaderView {
    pub block_hash: HashValue,
    /// Parent hash.
    pub parent_hash: HashValue,
    /// Block timestamp.
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
    pub timestamp: u64,
    /// Block number.
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
    pub sequence_number: u64,

    // The transaction payload in scs bytes.
    #[serde(
        serialize_with = "serialize_binary",
        deserialize_with = "deserialize_binary"
    )]
    pub payload: Vec<u8>,

    // Maximal total gas specified by wallet to spend for this transaction.
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
    pub max_gas_amount: u64,
    // Maximal price can be paid per gas.
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
    pub timestamp: u64,
    pub author: AccountAddress,
    pub author_auth_key: Option<AuthenticationKey>,
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
    pub uncles: u64,
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
    pub number: BlockNumber,
    pub chain_id: u8,
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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
        #[serde(
            deserialize_with = "deserialize_from_string",
            serialize_with = "serialize_to_string"
        )]
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
    #[serde(
        deserialize_with = "deserialize_from_string_opt",
        serialize_with = "serialize_to_string_opt"
    )]
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
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
    pub uncles: u64,
    /// sum(number of the block which contain uncle block - uncle parent block number).
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
    pub sum: u64,
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
    pub avg: u64,
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
    pub time_sum: u64,
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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
    #[serde(
        deserialize_with = "deserialize_from_string",
        serialize_with = "serialize_to_string"
    )]
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

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct StrView<T>(pub T);

impl<T> From<T> for StrView<T> {
    fn from(t: T) -> Self {
        Self(t)
    }
}

impl<T> Serialize for StrView<T>
where
    Self: ToString,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T> Deserialize<'de> for StrView<T>
where
    Self: FromStr,
    <Self as FromStr>::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;

        StrView::<T>::from_str(&s).map_err(D::Error::custom)
    }
}

pub type ModuleIdView = StrView<ModuleId>;
pub type TypeTagView = StrView<TypeTag>;
pub type StructTagView = StrView<StructTag>;
pub type TransactionArgumentView = StrView<TransactionArgument>;

impl std::fmt::Display for StrView<ModuleId> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for StrView<ModuleId> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split("::").collect();
        if parts.len() != 2 {
            anyhow::bail!("invalid module id");
        }
        let module_addr = parts[0].parse::<AccountAddress>()?;
        let module_name = Identifier::new(parts[1])?;
        Ok(Self(ModuleId::new(module_addr, module_name)))
    }
}
impl std::fmt::Display for StrView<TypeTag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for TypeTagView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let type_tag = parse_type_tag(s)?;
        Ok(Self(type_tag))
    }
}
impl std::fmt::Display for StrView<StructTag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for StructTagView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let type_tag = parse_type_tag(s)?;
        match type_tag {
            TypeTag::Struct(s) => Ok(Self(s)),
            t => anyhow::bail!("expect struct tag, actual: {}", t),
        }
    }
}
impl std::fmt::Display for StrView<TransactionArgument> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for TransactionArgumentView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let arg = parse_transaction_argument(s)?;
        Ok(Self(arg))
    }
}

impl std::fmt::Display for StrView<Vec<u8>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl FromStr for StrView<Vec<u8>> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode(
            s.strip_prefix("0x").unwrap_or_else(|| s),
        )?))
    }
}

impl std::fmt::Display for StrView<ByteCodeOrScriptName> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            ByteCodeOrScriptName::ByteCode(c) => write!(f, "0x{}", hex::encode(c)),
            ByteCodeOrScriptName::ScriptName(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for StrView<ByteCodeOrScriptName> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(match s.strip_prefix("0x") {
            Some(s) => ByteCodeOrScriptName::ByteCode(hex::decode(s)?),
            None => ByteCodeOrScriptName::ScriptName(s.to_string()),
        }))
    }
}

impl std::fmt::Display for StrView<AccountPublicKey> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_encoded_string().map_err(|_| std::fmt::Error)?
        )
    }
}

impl FromStr for StrView<AccountPublicKey> {
    type Err = CryptoMaterialError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AccountPublicKey::from_encoded_string(s).map(StrView)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContractCall {
    pub module_address: AccountAddress,
    pub module_name: String,
    pub func: String,
    pub type_args: Vec<TypeTagView>,
    pub args: Vec<TransactionArgumentView>,
}

#[derive(Debug, Clone)]
pub struct ConnectLocal;

impl ServiceRequest for ConnectLocal {
    type Response = RpcChannel;
}

#[cfg(test)]
mod test {
    use crate::types::{ContractCall, TransactionArgumentView, TypeTagView};
    use starcoin_vm_types::token::stc::stc_type_tag;
    use starcoin_vm_types::transaction_argument::TransactionArgument;

    #[test]
    fn test_view_of_type_tag() {
        let ty_tag = stc_type_tag();
        let s = serde_json::to_string(&TypeTagView::from(ty_tag.clone())).unwrap();
        println!("{}", &s);
        let ty_tag_view: TypeTagView = serde_json::from_str(s.as_str()).unwrap();
        assert_eq!(ty_tag_view.0, ty_tag);
    }

    #[test]
    fn test_view_of_transaction_arg() {
        let arg = TransactionArgument::U8(1);
        let s = serde_json::to_string(&TransactionArgumentView::from(arg.clone())).unwrap();
        println!("{}", &s);
        let view: TransactionArgumentView = serde_json::from_str(s.as_str()).unwrap();
        assert_eq!(view.0, arg);
    }

    #[test]
    fn test_deserialize() {
        let s = r#"
{
  "module_address": "0x0CC02653F9D7A62D07754D859B066BDE",
  "module_name": "T",
  "func": "A",
  "type_args": [ "0x42C4DDA17CC39AF459C20D09F6A82EDF::T::T"],
  "args": ["0xD6F8FAF8FA976104B8BA8C6F85DCF9E4"]
}        
        "#;
        let v = serde_json::from_str::<ContractCall>(s).unwrap();
        println!("{:?}", v);
    }
}
