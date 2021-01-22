// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use serde::{Deserialize, Serialize};
use starcoin_account_api::AccountInfo;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::{
    StrView, TransactionEventView, TransactionOutputAction, TransactionOutputView,
    TransactionVMStatus,
};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::{DepositEvent, MintEvent, WithdrawEvent};
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::language_storage::TypeTag;
use starcoin_vm_types::account_config::events::accept_token_payment::AcceptTokenEvent;
use starcoin_vm_types::account_config::{BlockRewardEvent, ProposalCreatedEvent, VoteChangedEvent};
use starcoin_vm_types::event::EventKey;
use starcoin_vm_types::move_resource::MoveResource;
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
    Mint { amount: u128, token_code: String },
    #[serde(rename = "receivedpayment")]
    ReceivedPayment {
        amount: u128,
        token_code: String,
        metadata: StrView<Vec<u8>>,
    },
    #[serde(rename = "sentpayment")]
    SentPayment {
        amount: u128,
        token_code: String,
        metadata: StrView<Vec<u8>>,
    },
    #[serde(rename = "upgrade")]
    Upgrade { write_set: StrView<Vec<u8>> },
    #[serde(rename = "newepoch")]
    NewEpoch { epoch: u64 },
    #[serde(rename = "newblock")]
    NewBlock {
        round: u64,
        proposer: StrView<Vec<u8>>,
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
    #[serde(rename = "block_reward_event")]
    BlockReward {
        block_number: u64,
        block_reward: u128,
        gas_fees: u128,
        miner: AccountAddress,
    },
    #[serde(rename = "unknown")]
    Unknown { type_tag: TypeTag, data: Vec<u8> },
}

impl EventDataView {
    pub fn new(event_type_tag: &TypeTag, event_data: &[u8]) -> anyhow::Result<Self> {
        if event_type_tag == &TypeTag::Struct(DepositEvent::struct_tag()) {
            if let Ok(received_event) = DepositEvent::try_from_bytes(event_data) {
                Ok(EventDataView::ReceivedPayment {
                    amount: received_event.amount(),
                    token_code: received_event.token_code().to_string(),
                    metadata: StrView(received_event.metadata().clone()),
                })
            } else {
                Err(format_err!("Unable to parse ReceivedPaymentEvent"))
            }
        } else if event_type_tag == &TypeTag::Struct(WithdrawEvent::struct_tag()) {
            if let Ok(sent_event) = WithdrawEvent::try_from_bytes(event_data) {
                Ok(EventDataView::SentPayment {
                    amount: sent_event.amount(),
                    token_code: sent_event.token_code().to_string(),
                    metadata: StrView(sent_event.metadata().clone()),
                })
            } else {
                Err(format_err!("Unable to parse SentPaymentEvent"))
            }
        } else if event_type_tag == &TypeTag::Struct(MintEvent::struct_tag()) {
            if let Ok(mint_event) = MintEvent::try_from_bytes(event_data) {
                Ok(EventDataView::Mint {
                    amount: mint_event.amount(),
                    token_code: mint_event.token_code().to_string(),
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
        } else if event_type_tag == &TypeTag::Struct(BlockRewardEvent::struct_tag()) {
            Ok(BlockRewardEvent::try_from_bytes(event_data)
                .map_err(|_| format_err!("Unable to parse {}", BlockRewardEvent::struct_tag()))?
                .into())
        } else {
            Ok(EventDataView::Unknown {
                type_tag: event_type_tag.clone(),
                data: event_data.to_vec(),
            })
        }
    }
}
impl From<BlockRewardEvent> for EventDataView {
    fn from(event: BlockRewardEvent) -> Self {
        EventDataView::BlockReward {
            block_number: event.block_number,

            block_reward: event.block_reward,
            gas_fees: event.gas_fees,
            miner: event.miner,
        }
    }
}
impl From<TransactionEventView> for EventView {
    fn from(event_view: TransactionEventView) -> Self {
        EventView {
            key: event_view.event_key,
            sequence_number: event_view.event_seq_number.0,
            data: EventDataView::new(&event_view.type_tag, &event_view.data).unwrap_or({
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

#[derive(Debug, Serialize)]
pub struct TranscationOutputView {
    pub write_set: Vec<TransactionOutputAction>,
    /// The list of events emitted during this transaction.
    pub events: Vec<EventView>,

    /// The amount of gas used during execution.
    pub gas_used: u64,

    /// The resource increment size
    pub delta_size: i64,

    /// The execution status.
    pub status: TransactionVMStatus,
}

impl From<TransactionOutputView> for TranscationOutputView {
    fn from(output: TransactionOutputView) -> Self {
        Self {
            write_set: output.write_set,
            events: output.events.into_iter().map(|e| e.into()).collect(),
            gas_used: output.gas_used.0,
            delta_size: output.delta_size.0,
            status: output.status,
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
pub struct MoveExplainView {
    pub category_code: u64,
    pub category_name: Option<String>,
    pub reason_code: u64,
    pub reason_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UncleInfo {
    pub uncle_view: starcoin_rpc_api::types::BlockHeaderView,
    pub uncle_parent_view: starcoin_rpc_api::types::BlockHeaderView,
    pub block_view: starcoin_rpc_api::types::BlockHeaderView,
}
