// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod contract_call;
pub mod pubsub;

pub use contract_call::*;
use jsonrpc_core_client::RpcChannel;
use scs::SCSCodec;
use serde::Deserialize;
use serde::Serialize;
use serde_helpers::serialize_binary;
use starcoin_crypto::HashValue;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::{BlockHeader, BlockNumber};
use starcoin_types::contract_event::{ContractEvent, ContractEventInfo};
use starcoin_types::event::EventKey;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::transaction::authenticator::{AuthenticationKey, TransactionAuthenticator};
use starcoin_types::transaction::RawUserTransaction;
use starcoin_types::vm_error::AbortLocation;
use starcoin_types::U256;
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::vm_status::KeptVMStatus;
use std::convert::TryFrom;
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockHeaderView {
    /// Parent hash.
    pub parent_hash: HashValue,
    /// Block timestamp.
    pub timestamp: u64,
    /// Block number.
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
    pub sequence_number: u64,

    // The transaction payload in scs bytes.
    #[serde(serialize_with = "serialize_binary")]
    pub payload: Vec<u8>,

    // Maximal total gas specified by wallet to spend for this transaction.
    pub max_gas_amount: u64,
    // Maximal price can be paid per gas.
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
    /// The raw transaction
    pub raw_txn: RawUserTransactionView,

    /// Public key and signature to authenticate
    pub authenticator: TransactionAuthenticator,
}
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BlockMetadataView {
    /// Parent block hash.
    pub parent_hash: HashValue,
    pub timestamp: u64,
    pub author: AccountAddress,
    pub author_auth_key: Option<AuthenticationKey>,
    pub uncles: u64,
    pub number: BlockNumber,
    pub chain_id: u8,
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

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TransactionView {
    block_id: HashValue,
    block_number: BlockNumber,
    hash: HashValue,
    transaction_index: u64,
    block_metadata: Option<BlockMetadataView>,
    user_transaction: Option<SignedUserTransactionView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BlockBodyView {
    Hashes(Vec<HashValue>),
    Full(Vec<SignedUserTransactionView>),
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockView {
    pub header: BlockHeaderView,
    pub body: BlockBodyView,
    pub uncles: Option<Vec<BlockHeaderView>>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionInfoView {
    block_id: HashValue,
    block_number: BlockNumber,
    /// The hash of this transaction.
    transaction_hash: HashValue,
    transaction_index: u64,
    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    event_root_hash: HashValue,

    /// The amount of gas used.
    gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    status: TransactionVMStatus,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionVMStatus {
    Executed,
    OutOfGas,
    MoveAbort {
        location: AbortLocation,
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
    pub transaction_index: Option<u64>,

    pub data: Vec<u8>,
    pub type_tags: TypeTag,
    pub event_key: EventKey,
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
            type_tags: info.event.type_tag().clone(),
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
        transaction_index: Option<u64>,
        contract_event: &ContractEvent,
    ) -> Self {
        Self {
            block_hash,
            block_number,
            transaction_hash,
            transaction_index,
            data: contract_event.event_data().to_vec(),
            type_tags: contract_event.type_tag().clone(),
            event_key: *contract_event.key(),
            event_seq_number: contract_event.sequence_number(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectLocal;

impl ServiceRequest for ConnectLocal {
    type Response = RpcChannel;
}
