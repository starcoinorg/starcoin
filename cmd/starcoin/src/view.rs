// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error};
use forkable_jellyfish_merkle::proof::SparseMerkleProof;
use serde::{Deserialize, Serialize, Serializer};
use starcoin_config::ChainNetwork;
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};
use starcoin_rpc_api::node::NodeInfo;
use starcoin_state_api::StateWithProof;
use starcoin_types::account_config::{MintEvent, ReceivedPaymentEvent, SentPaymentEvent};
use starcoin_types::block::{Block, BlockHeader};
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::peer_info::{PeerId, PeerInfo};
use starcoin_types::transaction::{TransactionInfo, TransactionStatus};
use starcoin_types::vm_error::StatusCode;
use starcoin_types::{account_address::AccountAddress, transaction::SignedUserTransaction, U256};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::transaction::TransactionOutput;
use starcoin_vm_types::write_set::WriteOp;
use starcoin_wallet_api::WalletAccount;
use std::collections::HashMap;

//TODO add a derive to auto generate View Object

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountWithStateView {
    pub account: WalletAccount,
    // hex encoded bytes
    pub auth_key_prefix: String,
    pub sequence_number: Option<u64>,
    pub balances: HashMap<String, u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountView {
    pub sequence_number: Option<u64>,
    pub balance: Option<u64>,
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
    pub transaction_hash: HashValue,
    pub state_root_hash: HashValue,
    pub event_root_hash: HashValue,
    pub events: Vec<EventView>,
    pub gas_used: u64,
    pub major_status: StatusCode,
}

impl From<TransactionInfo> for TransactionInfoView {
    fn from(tx: TransactionInfo) -> Self {
        TransactionInfoView {
            transaction_hash: tx.transaction_hash(),
            state_root_hash: tx.state_root_hash(),
            event_root_hash: tx.event_root_hash(),
            events: tx
                .events()
                .iter()
                .map(|event| EventView::from(event.clone()))
                .collect(),
            gas_used: tx.gas_used(),
            major_status: tx.major_status(),
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
        sender: BytesView,
        metadata: BytesView,
    },
    #[serde(rename = "sentpayment")]
    SentPayment {
        amount: AmountView,
        receiver: BytesView,
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
    #[serde(rename = "unknown")]
    Unknown {},
}

impl From<ContractEvent> for EventView {
    /// Tries to convert the provided byte array into Event Key.
    fn from(event: ContractEvent) -> EventView {
        let event_data = if event.type_tag() == &TypeTag::Struct(ReceivedPaymentEvent::struct_tag())
        {
            if let Ok(received_event) = ReceivedPaymentEvent::try_from_bytes(&event.event_data()) {
                let amount_view = AmountView::new(
                    received_event.amount(),
                    received_event.currency_code().as_str(),
                );
                Ok(EventDataView::ReceivedPayment {
                    amount: amount_view,
                    sender: BytesView::from(received_event.sender().as_ref()),
                    metadata: BytesView::from(received_event.metadata()),
                })
            } else {
                Err(format_err!("Unable to parse ReceivedPaymentEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(SentPaymentEvent::struct_tag()) {
            if let Ok(sent_event) = SentPaymentEvent::try_from_bytes(&event.event_data()) {
                let amount_view =
                    AmountView::new(sent_event.amount(), sent_event.currency_code().as_str());
                Ok(EventDataView::SentPayment {
                    amount: amount_view,
                    receiver: BytesView::from(sent_event.receiver().as_ref()),
                    metadata: BytesView::from(sent_event.metadata()),
                })
            } else {
                Err(format_err!("Unable to parse SentPaymentEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(MintEvent::struct_tag()) {
            if let Ok(mint_event) = MintEvent::try_from_bytes(&event.event_data()) {
                let amount_view =
                    AmountView::new(mint_event.amount(), mint_event.currency_code().as_str());
                Ok(EventDataView::Mint {
                    amount: amount_view,
                })
            } else {
                Err(format_err!("Unable to parse MintEvent"))
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
    pub amount: u64,
    pub currency: String,
}

impl AmountView {
    fn new(amount: u64, currency: &str) -> Self {
        Self {
            amount,
            currency: currency.to_string(),
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
    pub net: ChainNetwork,
}

impl From<NodeInfo> for NodeInfoView {
    fn from(node_info: NodeInfo) -> Self {
        Self {
            peer_info: node_info.peer_info.into(),
            self_address: node_info.self_address,
            net: node_info.net,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TranscationOutputView {
    pub write_set: Vec<(AccessPath, WriteOp)>,
    /// The list of events emitted during this transaction.
    pub events: Vec<ContractEvent>,

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
            write_set: write_set.into_iter().collect::<Vec<_>>(),
            events,
            gas_used,
            delta_size,
            status,
        }
    }
}

#[derive(Debug)]
pub enum ExecuteResultView {
    DryRunOutput(TranscationOutputView),
    RunOutput(ExecutionOutputView),
}

impl serde::Serialize for ExecuteResultView {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            ExecuteResultView::DryRunOutput(o) => o.serialize(serializer),
            ExecuteResultView::RunOutput(o) => o.serialize(serializer),
        }
    }
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
