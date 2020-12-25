// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error};
use forkable_jellyfish_merkle::proof::SparseMerkleProof;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};
use starcoin_account_api::AccountInfo;
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};
use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use starcoin_rpc_api::types::{StrView, StructTagView, TransactionEventView};
use starcoin_state_api::StateWithProof;
use starcoin_types::account_config::{DepositEvent, MintEvent, WithdrawEvent};
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::transaction::{TransactionInfo, TransactionStatus};
use starcoin_types::{account_address::AccountAddress, transaction::SignedUserTransaction};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::events::accept_token_payment::AcceptTokenEvent;
use starcoin_vm_types::account_config::{ProposalCreatedEvent, VoteChangedEvent};
use starcoin_vm_types::event::EventKey;
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

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct EventView {
    pub key: EventKey,
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
    #[serde(rename = "accept_token")]
    AcceptToken { token_code: String },
    #[serde(rename = "unknown")]
    Unknown { type_tag: TypeTag, data: Vec<u8> },
}

impl EventDataView {
    pub fn new(event_type_tag: &TypeTag, event_data: &[u8]) -> anyhow::Result<Self> {
        if event_type_tag == &TypeTag::Struct(DepositEvent::struct_tag()) {
            if let Ok(received_event) = DepositEvent::try_from_bytes(event_data) {
                let amount_view = AmountView::new(
                    received_event.amount(),
                    received_event.token_code().to_string(),
                );
                Ok(EventDataView::ReceivedPayment {
                    amount: amount_view,
                    metadata: BytesView::from(received_event.metadata()),
                })
            } else {
                Err(format_err!("Unable to parse ReceivedPaymentEvent"))
            }
        } else if event_type_tag == &TypeTag::Struct(WithdrawEvent::struct_tag()) {
            if let Ok(sent_event) = WithdrawEvent::try_from_bytes(event_data) {
                let amount_view =
                    AmountView::new(sent_event.amount(), sent_event.token_code().to_string());
                Ok(EventDataView::SentPayment {
                    amount: amount_view,
                    metadata: BytesView::from(sent_event.metadata()),
                })
            } else {
                Err(format_err!("Unable to parse SentPaymentEvent"))
            }
        } else if event_type_tag == &TypeTag::Struct(MintEvent::struct_tag()) {
            if let Ok(mint_event) = MintEvent::try_from_bytes(event_data) {
                let amount_view =
                    AmountView::new(mint_event.amount(), mint_event.token_code().to_string());
                Ok(EventDataView::Mint {
                    amount: amount_view,
                })
            } else {
                Err(format_err!("Unable to parse MintEvent"))
            }
        } else if event_type_tag == &TypeTag::Struct(ProposalCreatedEvent::struct_tag()) {
            if let Ok(event) = ProposalCreatedEvent::try_from_bytes(event_data) {
                Ok(EventDataView::ProposalCreated {
                    proposal_id: event.proposal_id,
                    proposer: format!("{}", event.proposer),
                })
            } else {
                Err(format_err!("Unable to parse ProposalCreatedEvent"))
            }
        } else if event_type_tag == &TypeTag::Struct(VoteChangedEvent::struct_tag()) {
            if let Ok(event) = VoteChangedEvent::try_from_bytes(event_data) {
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
        } else if event_type_tag == &TypeTag::Struct(AcceptTokenEvent::struct_tag()) {
            if let Ok(event) = AcceptTokenEvent::try_from_bytes(event_data) {
                Ok(EventDataView::AcceptToken {
                    token_code: event.token_code().to_string(),
                })
            } else {
                Err(format_err!("Unable to parse VoteChangedEvent"))
            }
        } else {
            Ok(EventDataView::Unknown {
                type_tag: event_type_tag.clone(),
                data: event_data.to_vec(),
            })
        }
    }
}

impl From<TransactionEventView> for EventView {
    fn from(event_view: TransactionEventView) -> Self {
        EventView {
            key: event_view.event_key,
            sequence_number: event_view.event_seq_number,
            data: EventDataView::new(&event_view.type_tag, &event_view.data).unwrap_or_else(|_| {
                EventDataView::Unknown {
                    data: event_view.data,
                    type_tag: event_view.type_tag,
                }
            }),
        }
    }
}

impl From<ContractEvent> for EventView {
    /// Tries to convert the provided byte array into Event Key.
    fn from(event: ContractEvent) -> EventView {
        let event_data = EventDataView::new(event.type_tag(), event.event_data());
        EventView {
            key: *event.key(),
            sequence_number: event.sequence_number(),
            data: event_data.unwrap_or(EventDataView::Unknown {
                data: event.event_data().to_vec(),
                type_tag: event.type_tag().clone(),
            }),
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
    fn new(amount: u128, token_code: String) -> Self {
        Self { amount, token_code }
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

#[derive(Debug, Serialize)]
pub struct MoveExplainView {
    pub category_code: u64,
    pub category_name: Option<String>,
    pub reason_code: u64,
    pub reason_name: Option<String>,
}

pub fn serialize_bytes_to_hex<S>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&hex::encode(bytes))
}

#[derive(Debug, Clone, Serialize)]
pub struct UncleInfo {
    pub uncle_view: starcoin_rpc_api::types::BlockHeaderView,
    pub uncle_parent_view: starcoin_rpc_api::types::BlockHeaderView,
    pub block_view: starcoin_rpc_api::types::BlockHeaderView,
}

/// only used to display in cli.
#[derive(Serialize)]
pub struct MoveStructView {
    is_resource: bool,
    type_: StructTagView,
    value: HashMap<Identifier, MoveValueView>,
}

impl From<AnnotatedMoveStruct> for MoveStructView {
    fn from(s: AnnotatedMoveStruct) -> Self {
        let AnnotatedMoveStruct {
            is_resource,
            type_,
            value,
        } = s;
        MoveStructView {
            is_resource,
            type_: StrView(type_),
            value: value
                .into_iter()
                .map(|(f, v)| (f, MoveValueView(v)))
                .collect(),
        }
    }
}

/// only used to display in cli.
struct MoveValueView(AnnotatedMoveValue);
impl Serialize for MoveValueView {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            AnnotatedMoveValue::U8(d) => d.serialize(serializer),
            AnnotatedMoveValue::U64(d) => d.serialize(serializer),
            AnnotatedMoveValue::U128(d) => d.serialize(serializer),
            AnnotatedMoveValue::Bool(d) => d.serialize(serializer),
            AnnotatedMoveValue::Address(d) => d.serialize(serializer),
            AnnotatedMoveValue::Vector(v) => {
                let mut serializer = serializer.serialize_seq(Some(v.len()))?;
                for elem in v {
                    serializer.serialize_element(&MoveValueView(elem.clone()))?;
                }
                serializer.end()
            }
            AnnotatedMoveValue::Bytes(bytes) => {
                format!("0x{}", hex::encode(bytes)).serialize(serializer)
            }
            AnnotatedMoveValue::Struct(s) => {
                let view: MoveStructView = s.clone().into();
                view.serialize(serializer)
            }
        }
    }
}
