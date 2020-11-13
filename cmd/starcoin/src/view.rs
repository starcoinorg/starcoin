// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error};
use forkable_jellyfish_merkle::proof::SparseMerkleProof;
use serde::{Deserialize, Serialize, Serializer};
use starcoin_account_api::AccountInfo;
use starcoin_config::ChainNetworkID;
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};
use starcoin_rpc_api::node::NodeInfo;
use starcoin_state_api::StateWithProof;
use starcoin_types::account_config::{DepositEvent, MintEvent, WithdrawEvent};
use starcoin_types::block::{Block, BlockHeader};
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::peer_info::{PeerId, PeerInfo};
use starcoin_types::transaction::{TransactionInfo, TransactionStatus};
use starcoin_types::{account_address::AccountAddress, transaction::SignedUserTransaction, U256};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::{ProposalCreatedEvent, VoteChangedEvent};
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::transaction::TransactionOutput;
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::write_set::WriteOp;
use std::collections::HashMap;

//TODO add a derive to auto generate View Object

#[derive(Debug, Serialize, Deserialize)]
pub struct StringView {
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountWithStateView {
    pub account: AccountInfo,
    pub auth_key: String,
    pub sequence_number: Option<u64>,
    pub balances: HashMap<String, u128>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountView {
    pub sequence_number: Option<u64>,
    pub balance: Option<u128>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateWithProofView {
    pub state: String,
    pub account_proof: SparseMerkleProof,
    pub account_state_proof: SparseMerkleProof,
}
impl From<StateWithProof> for StateWithProofView {
    fn from(state_proof: StateWithProof) -> Self {
        let account_state = hex::encode(state_proof.state.unwrap());
        Self {
            state: account_state,
            account_proof: state_proof.proof.account_proof,
            account_state_proof: state_proof.proof.account_state_proof,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockHeaderView {
    pub parent_hash: HashValue,
    pub number: u64,
    pub id: HashValue,
    pub author: AccountAddress,
    pub accumulator_root: HashValue,
    pub state_root: HashValue,
    pub gas_used: u64,
    pub time: u64,
}

impl From<Block> for BlockHeaderView {
    fn from(block: Block) -> Self {
        BlockHeaderView::from(block.header)
    }
}

impl From<BlockHeader> for BlockHeaderView {
    fn from(header: BlockHeader) -> Self {
        Self {
            parent_hash: header.parent_hash,
            number: header.number,
            id: header.id(),
            author: header.author,
            accumulator_root: header.accumulator_root,
            state_root: header.state_root,
            gas_used: header.gas_used,
            time: header.timestamp,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionView {
    pub id: HashValue,
    pub sender: AccountAddress,
    pub sequence_number: u64,
    pub gas_unit_price: u64,
    pub max_gas_amount: u64,
}

impl From<SignedUserTransaction> for TransactionView {
    fn from(txn: SignedUserTransaction) -> Self {
        Self {
            id: txn.raw_txn().crypto_hash(),
            sender: txn.sender(),
            sequence_number: txn.sequence_number(),
            gas_unit_price: txn.gas_unit_price(),
            max_gas_amount: txn.max_gas_amount(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionInfoView {
    pub txn_info_id: HashValue,
    pub transaction_hash: HashValue,
    pub state_root_hash: HashValue,
    pub event_root_hash: HashValue,
    pub gas_used: u64,
    pub status: KeptVMStatus,
}

impl From<TransactionInfo> for TransactionInfoView {
    fn from(tx: TransactionInfo) -> Self {
        TransactionInfoView {
            txn_info_id: tx.id(),
            transaction_hash: tx.transaction_hash(),
            state_root_hash: tx.state_root_hash(),
            event_root_hash: tx.event_root_hash(),
            gas_used: tx.gas_used(),
            status: tx.status().clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventView {
    pub key: BytesView,
    pub sequence_number: u64,
    pub data: EventDataView,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum EventDataView {
    #[serde(rename = "mint")]
    Mint { amount: AmountView },
    #[serde(rename = "receivedpayment")]
    ReceivedPayment {
        amount: AmountView,
        metadata: BytesView,
    },
    #[serde(rename = "sentpayment")]
    SentPayment {
        amount: AmountView,
        metadata: BytesView,
    },
    #[serde(rename = "upgrade")]
    Upgrade { write_set: BytesView },
    #[serde(rename = "newepoch")]
    NewEpoch { epoch: u64 },
    #[serde(rename = "newblock")]
    NewBlock {
        round: u64,
        proposer: BytesView,
        proposed_time: u64,
    },
    #[serde(rename = "proposal_created")]
    ProposalCreated { proposal_id: u64, proposer: String },
    #[serde(rename = "vote_changed")]
    VoteChanged {
        proposal_id: u64,
        proposer: String,
        voter: String,
        agree: bool,
        vote: u128,
    },
    #[serde(rename = "unknown")]
    Unknown {},
}

impl From<ContractEvent> for EventView {
    /// Tries to convert the provided byte array into Event Key.
    fn from(event: ContractEvent) -> EventView {
        let event_data = if event.type_tag() == &TypeTag::Struct(DepositEvent::struct_tag()) {
            if let Ok(received_event) = DepositEvent::try_from_bytes(&event.event_data()) {
                let amount_view = AmountView::new(
                    received_event.amount(),
                    received_event.currency_code().as_str(),
                );
                Ok(EventDataView::ReceivedPayment {
                    amount: amount_view,
                    metadata: BytesView::from(received_event.metadata()),
                })
            } else {
                Err(format_err!("Unable to parse ReceivedPaymentEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(WithdrawEvent::struct_tag()) {
            if let Ok(sent_event) = WithdrawEvent::try_from_bytes(&event.event_data()) {
                let amount_view =
                    AmountView::new(sent_event.amount(), sent_event.currency_code().as_str());
                Ok(EventDataView::SentPayment {
                    amount: amount_view,
                    metadata: BytesView::from(sent_event.metadata()),
                })
            } else {
                Err(format_err!("Unable to parse SentPaymentEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(MintEvent::struct_tag()) {
            if let Ok(mint_event) = MintEvent::try_from_bytes(&event.event_data()) {
                let amount_view =
                    AmountView::new(mint_event.amount(), mint_event.token_code().as_str());
                Ok(EventDataView::Mint {
                    amount: amount_view,
                })
            } else {
                Err(format_err!("Unable to parse MintEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(ProposalCreatedEvent::struct_tag()) {
            if let Ok(event) = ProposalCreatedEvent::try_from_bytes(&event.event_data()) {
                Ok(EventDataView::ProposalCreated {
                    proposal_id: event.proposal_id,
                    proposer: format!("{}", event.proposer),
                })
            } else {
                Err(format_err!("Unable to parse ProposalCreatedEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(VoteChangedEvent::struct_tag()) {
            if let Ok(event) = VoteChangedEvent::try_from_bytes(&event.event_data()) {
                Ok(EventDataView::VoteChanged {
                    proposal_id: event.proposal_id,
                    proposer: format!("{}", event.proposer),
                    voter: format!("{}", event.voter),
                    agree: event.agree,
                    vote: event.vote,
                })
            } else {
                Err(format_err!("Unable to parse VoteChangedEvent"))
            }
        } else {
            Err(format_err!("Unknown events"))
        };

        EventView {
            key: BytesView::from(event.key().as_bytes()),
            sequence_number: event.sequence_number(),
            data: event_data.unwrap_or(EventDataView::Unknown {}),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct BytesView(pub String);

impl BytesView {
    pub fn into_bytes(self) -> Result<Vec<u8>, Error> {
        Ok(hex::decode(self.0)?)
    }
}

impl From<&[u8]> for BytesView {
    fn from(bytes: &[u8]) -> Self {
        Self(hex::encode(bytes))
    }
}

impl From<&Vec<u8>> for BytesView {
    fn from(bytes: &Vec<u8>) -> Self {
        Self(hex::encode(bytes))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AmountView {
    pub amount: u128,
    pub token_code: String,
}

impl AmountView {
    fn new(amount: u128, token_code: &str) -> Self {
        Self {
            amount,
            token_code: token_code.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerInfoView {
    pub peer_id: PeerId,
    pub latest_header: BlockHeaderView,
    pub total_difficulty: U256,
}

impl From<PeerInfo> for PeerInfoView {
    fn from(peer_info: PeerInfo) -> Self {
        Self {
            peer_id: peer_info.peer_id,
            latest_header: peer_info.latest_header.into(),
            total_difficulty: peer_info.total_difficulty,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfoView {
    pub peer_info: PeerInfoView,
    pub self_address: String,
    pub net: ChainNetworkID,
    pub now: u64,
}

impl From<NodeInfo> for NodeInfoView {
    fn from(node_info: NodeInfo) -> Self {
        Self {
            peer_info: node_info.peer_info.into(),
            self_address: node_info.self_address,
            net: node_info.net,
            now: node_info.now_seconds,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct TranscationOutputView {
    pub write_set: Vec<(AccessPathView, WriteOpView)>,
    /// The list of events emitted during this transaction.
    pub events: Vec<EventView>,

    /// The amount of gas used during execution.
    pub gas_used: u64,

    /// The resource increment size
    pub delta_size: i64,

    /// The execution status.
    pub status: TransactionStatus,
}

impl From<TransactionOutput> for TranscationOutputView {
    fn from(output: TransactionOutput) -> Self {
        let (write_set, events, gas_used, delta_size, status) = output.into_inner();
        Self {
            write_set: write_set
                .into_iter()
                .map(|(ap, w)| (ap.into(), w.into()))
                .collect::<Vec<_>>(),
            events: events.into_iter().map(|e| e.into()).collect(),
            gas_used,
            delta_size,
            status,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ExecuteResultView {
    DryRun(TranscationOutputView),
    Run(ExecutionOutputView),
}

#[derive(Serialize, Debug, Clone, Copy)]
pub struct ExecutionOutputView {
    pub txn_hash: HashValue,
    pub block_number: Option<u64>,
    pub block_id: Option<HashValue>,
}

impl ExecutionOutputView {
    pub fn new(txn_hash: HashValue) -> Self {
        Self {
            txn_hash,
            block_number: None,
            block_id: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AccessPathView {
    address: AccountAddress,
    ty: u8,
    #[serde(serialize_with = "serialize_bytes_to_hex")]
    hash: Vec<u8>,
    suffix: String,
}

impl From<AccessPath> for AccessPathView {
    fn from(ap: AccessPath) -> Self {
        Self {
            address: ap.address,
            ty: ap.path[0],
            hash: ap.path[1..=HashValue::LENGTH].to_vec(),
            suffix: String::from_utf8_lossy(&ap.path[1 + HashValue::LENGTH..]).to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WriteOpView {
    Deletion,
    Value(#[serde(serialize_with = "serialize_bytes_to_hex")] Vec<u8>),
}

impl From<WriteOp> for WriteOpView {
    fn from(op: WriteOp) -> Self {
        match op {
            WriteOp::Deletion => WriteOpView::Deletion,
            WriteOp::Value(v) => WriteOpView::Value(v),
        }
    }
}

pub fn serialize_bytes_to_hex<S>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&hex::encode(bytes))
}
