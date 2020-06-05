// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use forkable_jellyfish_merkle::proof::SparseMerkleProof;
use serde::{Deserialize, Serialize};
use starcoin_config::ChainNetwork;
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};
use starcoin_rpc_api::node::NodeInfo;
use starcoin_state_api::StateWithProof;
use starcoin_types::block::{Block, BlockHeader};
use starcoin_types::peer_info::{PeerId, PeerInfo};
use starcoin_types::{account_address::AccountAddress, transaction::SignedUserTransaction, U256};
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
