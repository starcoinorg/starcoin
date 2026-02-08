// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use jsonrpsee::{
    core::{
        client::{ClientT, Subscription, SubscriptionClientT},
        params::{ArrayParams, ObjectParams},
    },
    rpc_params,
};
use network_api::PeerStrategy;
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::peer_id::PeerId;
use network_types::peer_info::Multiaddr;
use serde_json::Value;
use starcoin_abi_types::{FunctionABI, ModuleABI, StructInstantiation};
use starcoin_account_api::AccountInfo;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::multi_types::MultiSignedUserTransactionView;
use starcoin_rpc_api::chain::{
    GetBlockOption, GetBlocksOption, GetEventOption, GetTransactionOption,
};
use starcoin_rpc_api::node::NodeInfo;
use starcoin_rpc_api::state::{
    GetCodeOption, GetResourceOption, ListCodeOption, ListResourceOption,
};
use starcoin_rpc_api::types::pubsub::{EventFilter, EventFilterV2, EventParams, EventParamsV2, Kind};
use starcoin_rpc_api::types::{
    AccountStateSetView, AnnotatedMoveStructView, BlockHeaderView, BlockInfoView, BlockView,
    ChainId, ChainInfoView, CodeView, ContractCall, DecodedMoveValue, DryRunOutputView,
    DryRunTransactionRequest, FactoryAction, FunctionIdView, ListCodeView, ListResourceView,
    MintedBlockView, ModuleIdView, MultiStateView, PeerInfoView, ResourceView, SignedMessageView,
    StateWithProofView, StateWithTableItemProofView, StrView, StructTagView, SyncStatusView,
    TransactionEventResponse, TransactionEventView, TransactionInfoView, TransactionInfoViewEnum,
    TransactionInfoWithProofView, TransactionRequest, TransactionView,
};
use starcoin_service_registry::{ServiceInfo, ServiceStatus};
use starcoin_sync_api::{PeerScoreResponse, SyncProgressReport};
use starcoin_txpool_api::TxPoolStatus;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::block::BlockNumber;
use starcoin_types::multi_access_path::MultiAccessPath;
use starcoin_types::sign_message::SigningMessage;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm2_rpc_api::block_info_view2::BlockInfoView2;
use starcoin_vm2_rpc_api::transaction_view2::TransactionView2;
use starcoin_vm2_account_api::AccountInfo as AccountInfo2;
use starcoin_vm2_types::account_address::AccountAddress as AccountAddress2;
use starcoin_vm2_types::view::TransactionInfoView as TransactionInfoView2;
use starcoin_vm2_vm_types::transaction::SignedUserTransaction as SignedUserTransaction2;
use starcoin_vm_types::language_storage::{ModuleId, StructTag};
use starcoin_vm_types::state_store::table::TableHandle;
use starcoin_vm_types::token::token_code::TokenCode;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub type RpcChannel = Arc<jsonrpsee::async_client::Client>;
pub type RpcError = anyhow::Error;

pub async fn connect_ipc(sock_path: PathBuf) -> Result<RpcChannel, RpcError> {
    let client = starcoin_rpc_ipc::client::IpcClientBuilder::default()
        .build(
            sock_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("invalid ipc path"))?,
        )
        .await?;
    Ok(Arc::new(client))
}

pub async fn connect_ws(url: &str) -> Result<RpcChannel, RpcError> {
    Err(anyhow::anyhow!(
        "websocket rpc is not enabled in this build: {url}"
    ))
}

#[derive(Clone)]
pub struct RawClient {
    inner: RpcChannel,
}

impl From<RpcChannel> for RawClient {
    fn from(inner: RpcChannel) -> Self {
        Self { inner }
    }
}

impl RawClient {
    pub async fn call_method(self, api: &str, params: Value) -> Result<Value, RpcError> {
        match params {
            Value::Null => self.inner.request(api, rpc_params![]).await.map_err(Into::into),
            Value::Array(values) => {
                let mut array = ArrayParams::new();
                for v in values {
                    array.insert(v)?;
                }
                self.inner.request(api, array).await.map_err(Into::into)
            }
            Value::Object(values) => {
                let mut object = ObjectParams::new();
                for (k, v) in values {
                    object.insert(&k, v)?;
                }
                self.inner.request(api, object).await.map_err(Into::into)
            }
            other => self.inner.request(api, rpc_params![other]).await.map_err(Into::into),
        }
    }
}

macro_rules! def_client {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name {
            inner: RpcChannel,
        }
        impl From<RpcChannel> for $name {
            fn from(inner: RpcChannel) -> Self {
                Self { inner }
            }
        }
    };
}

def_client!(NodeClient);
def_client!(NodeManagerClient);
def_client!(TxPoolClient);
def_client!(AccountClient);
def_client!(AccountClient2);
def_client!(StateClient);
def_client!(StateClient2);
def_client!(DebugClient);
def_client!(ChainClient);
def_client!(ContractClient);
def_client!(ContractClient2);
def_client!(MinerClient);
def_client!(SyncManagerClient);
def_client!(NetworkManagerClient);

#[derive(Clone)]
pub struct PubSubClient {
    inner: RpcChannel,
}

impl From<RpcChannel> for PubSubClient {
    fn from(inner: RpcChannel) -> Self {
        Self { inner }
    }
}

impl NodeClient {
    pub async fn status(self) -> Result<bool, RpcError> {
        self.inner.request("node.status", rpc_params![]).await.map_err(Into::into)
    }
    pub async fn info(self) -> Result<NodeInfo, RpcError> {
        self.inner.request("node.info", rpc_params![]).await.map_err(Into::into)
    }
    pub async fn peers(self) -> Result<Vec<PeerInfoView>, RpcError> {
        self.inner.request("node.peers", rpc_params![]).await.map_err(Into::into)
    }
    pub async fn metrics(self) -> Result<HashMap<String, String>, RpcError> {
        self.inner.request("node.metrics", rpc_params![]).await.map_err(Into::into)
    }
}

impl NodeManagerClient {
    pub async fn list_service(self) -> Result<Vec<ServiceInfo>, RpcError> {
        self.inner.request("node_manager.list_service", rpc_params![]).await.map_err(Into::into)
    }
    pub async fn start_service(self, service_name: String) -> Result<(), RpcError> {
        self.inner.request("node_manager.start_service", rpc_params![service_name]).await.map_err(Into::into)
    }
    pub async fn check_service(self, service_name: String) -> Result<ServiceStatus, RpcError> {
        self.inner.request("node_manager.check_service", rpc_params![service_name]).await.map_err(Into::into)
    }
    pub async fn stop_service(self, service_name: String) -> Result<(), RpcError> {
        self.inner.request("node_manager.stop_service", rpc_params![service_name]).await.map_err(Into::into)
    }
    pub async fn shutdown_system(self) -> Result<(), RpcError> {
        self.inner.request("node_manager.shutdown_system", rpc_params![]).await.map_err(Into::into)
    }
    pub async fn reset_to_block(self, block_hash: HashValue) -> Result<(), RpcError> {
        self.inner.request("node_manager.reset_to_block", rpc_params![block_hash]).await.map_err(Into::into)
    }
    pub async fn re_execute_block(self, block_hash: HashValue) -> Result<(), RpcError> {
        self.inner.request("node_manager.re_execute_block", rpc_params![block_hash]).await.map_err(Into::into)
    }
    pub async fn delete_block(self, block_hash: HashValue) -> Result<(), RpcError> {
        self.inner.request("node_manager.delete_block", rpc_params![block_hash]).await.map_err(Into::into)
    }
    pub async fn delete_failed_block(self, block_hash: HashValue) -> Result<(), RpcError> {
        self.inner.request("node_manager.delete_failed_block", rpc_params![block_hash]).await.map_err(Into::into)
    }
}

impl TxPoolClient {
    pub async fn submit_transaction(self, txn: SignedUserTransaction) -> Result<HashValue, RpcError> {
        self.inner.request("txpool.submit_transaction", rpc_params![txn]).await.map_err(Into::into)
    }
    pub async fn submit_transactions(self, txns: Vec<SignedUserTransaction>) -> Result<Vec<HashValue>, RpcError> {
        self.inner.request("txpool.submit_transactions", rpc_params![txns]).await.map_err(Into::into)
    }
    pub async fn submit_hex_transaction(self, txn: String) -> Result<HashValue, RpcError> {
        self.inner.request("txpool.submit_hex_transaction", rpc_params![txn]).await.map_err(Into::into)
    }
    pub async fn pending_txn_multi(self, txn_hash: HashValue) -> Result<Option<MultiSignedUserTransactionView>, RpcError> {
        self.inner.request("txpool.pending_txn_multi", rpc_params![txn_hash]).await.map_err(Into::into)
    }
    pub async fn pending_txns_multi(self, sender: AccountAddress, max_len: Option<u32>) -> Result<Vec<MultiSignedUserTransactionView>, RpcError> {
        self.inner.request("txpool.pending_txns_of_sender_multi", rpc_params![sender, max_len]).await.map_err(Into::into)
    }
    pub async fn next_sequence_number(self, address: AccountAddress) -> Result<Option<u64>, RpcError> {
        self.inner.request("txpool.next_sequence_number", rpc_params![address]).await.map_err(Into::into)
    }
    pub async fn next_sequence_number_in_batch(self,
        addresses: Vec<AccountAddress>,
    ) -> Result<Option<Vec<(AccountAddress, Option<u64>)>>, RpcError> {
        self.inner.request("txpool.next_sequence_number_in_batch", rpc_params![addresses]).await.map_err(Into::into)
    }
    pub async fn state(self) -> Result<TxPoolStatus, RpcError> {
        self.inner.request("txpool.state", rpc_params![]).await.map_err(Into::into)
    }
    pub async fn submit_transaction2(self, txn: SignedUserTransaction2) -> Result<HashValue, RpcError> {
        self.inner.request("txpool.submit_transaction2", rpc_params![txn]).await.map_err(Into::into)
    }
    pub async fn submit_hex_transaction2(self, txn: String) -> Result<HashValue, RpcError> {
        self.inner.request("txpool.submit_hex_transaction2", rpc_params![txn]).await.map_err(Into::into)
    }
    pub async fn next_sequence_number2(self, address: AccountAddress2) -> Result<Option<u64>, RpcError> {
        self.inner.request("txpool.next_sequence_number2", rpc_params![address]).await.map_err(Into::into)
    }
}

impl AccountClient {
    pub async fn default(self) -> Result<Option<AccountInfo>, RpcError> { self.inner.request("account.default", rpc_params![]).await.map_err(Into::into) }
    pub async fn set_default_account(self, addr: AccountAddress) -> Result<AccountInfo, RpcError> { self.inner.request("account.set_default_account", rpc_params![addr]).await.map_err(Into::into) }
    pub async fn create(self, password: String) -> Result<AccountInfo, RpcError> { self.inner.request("account.create", rpc_params![password]).await.map_err(Into::into) }
    pub async fn list(self) -> Result<Vec<AccountInfo>, RpcError> { self.inner.request("account.list", rpc_params![]).await.map_err(Into::into) }
    pub async fn get(self, address: AccountAddress) -> Result<Option<AccountInfo>, RpcError> { self.inner.request("account.get", rpc_params![address]).await.map_err(Into::into) }
    pub async fn sign_txn(self, raw_txn: RawUserTransaction, signer: AccountAddress) -> Result<SignedUserTransaction, RpcError> { self.inner.request("account.sign_txn", rpc_params![raw_txn, signer]).await.map_err(Into::into) }
    pub async fn sign_txn_request(self, txn_request: TransactionRequest) -> Result<String, RpcError> { self.inner.request("account.sign_txn_request", rpc_params![txn_request]).await.map_err(Into::into) }
    pub async fn sign_txn_in_batch(self, raw_txns: Vec<RawUserTransaction>) -> Result<Vec<SignedUserTransaction>, RpcError> { self.inner.request("account.sign_txn_in_batch", rpc_params![raw_txns]).await.map_err(Into::into) }
    pub async fn sign(self, signer: AccountAddress, message: SigningMessage) -> Result<SignedMessageView, RpcError> { self.inner.request("account.sign", rpc_params![signer, message]).await.map_err(Into::into) }
    pub async fn change_account_password(self, address: AccountAddress, new_password: String) -> Result<AccountInfo, RpcError> { self.inner.request("account.change_password", rpc_params![address, new_password]).await.map_err(Into::into) }
    pub async fn lock(self, address: AccountAddress) -> Result<AccountInfo, RpcError> { self.inner.request("account.lock", rpc_params![address]).await.map_err(Into::into) }
    pub async fn unlock(self, address: AccountAddress, password: String, duration: Option<u32>) -> Result<AccountInfo, RpcError> { self.inner.request("account.unlock", rpc_params![address, password, duration]).await.map_err(Into::into) }
    pub async fn unlock_in_batch(self, batch: Vec<(AccountAddress, String)>, duration: Option<u32>) -> Result<Vec<AccountInfo>, RpcError> { self.inner.request("account.unlock_in_batch", rpc_params![batch, duration]).await.map_err(Into::into) }
    pub async fn export(self, address: AccountAddress, password: String) -> Result<Vec<u8>, RpcError> { self.inner.request("account.export", rpc_params![address, password]).await.map_err(Into::into) }
    pub async fn import(self, address: AccountAddress, private_key: StrView<Vec<u8>>, password: String) -> Result<AccountInfo, RpcError> { self.inner.request("account.import", rpc_params![address, private_key, password]).await.map_err(Into::into) }
    pub async fn import_readonly(self, address: AccountAddress, public_key: StrView<Vec<u8>>) -> Result<AccountInfo, RpcError> { self.inner.request("account.import_readonly", rpc_params![address, public_key]).await.map_err(Into::into) }
    pub async fn accepted_tokens(self, address: AccountAddress) -> Result<Vec<TokenCode>, RpcError> { self.inner.request("account.accepted_tokens", rpc_params![address]).await.map_err(Into::into) }
    pub async fn remove(self, address: AccountAddress, password: Option<String>) -> Result<AccountInfo, RpcError> { self.inner.request("account.remove", rpc_params![address, password]).await.map_err(Into::into) }
}

impl AccountClient2 {
    pub async fn default(self) -> Result<Option<AccountInfo2>, RpcError> { self.inner.request("account2.default", rpc_params![]).await.map_err(Into::into) }
    pub async fn set_default_account(self, addr: AccountAddress2) -> Result<AccountInfo2, RpcError> { self.inner.request("account2.set_default_account", rpc_params![addr]).await.map_err(Into::into) }
    pub async fn list(self) -> Result<Vec<AccountInfo2>, RpcError> { self.inner.request("account2.list", rpc_params![]).await.map_err(Into::into) }
    pub async fn get(self, address: AccountAddress2) -> Result<Option<AccountInfo2>, RpcError> { self.inner.request("account2.get", rpc_params![address]).await.map_err(Into::into) }
    pub async fn sign(self, signer: AccountAddress2, message: starcoin_vm2_vm_types::sign_message::SigningMessage) -> Result<starcoin_vm2_types::view::SignedMessageView, RpcError> { self.inner.request("account2.sign", rpc_params![signer, message]).await.map_err(Into::into) }
    pub async fn sign_txn_request(self, txn_request: starcoin_vm2_types::view::TransactionRequest) -> Result<String, RpcError> { self.inner.request("account2.sign_txn_request", rpc_params![txn_request]).await.map_err(Into::into) }
    pub async fn unlock(self, address: AccountAddress2, password: String, duration: Option<u32>) -> Result<AccountInfo2, RpcError> { self.inner.request("account2.unlock", rpc_params![address, password, duration]).await.map_err(Into::into) }
    pub async fn lock(self, address: AccountAddress2) -> Result<AccountInfo2, RpcError> { self.inner.request("account2.lock", rpc_params![address]).await.map_err(Into::into) }
    pub async fn import(self, address: AccountAddress2, private_key: starcoin_vm2_types::view::StrView<Vec<u8>>, password: String) -> Result<AccountInfo2, RpcError> { self.inner.request("account2.import", rpc_params![address, private_key, password]).await.map_err(Into::into) }
    pub async fn import_readonly(self, address: AccountAddress2, public_key: starcoin_vm2_types::view::StrView<Vec<u8>>) -> Result<AccountInfo2, RpcError> { self.inner.request("account2.import_readonly", rpc_params![address, public_key]).await.map_err(Into::into) }
    pub async fn export(self, address: AccountAddress2, password: String) -> Result<Vec<u8>, RpcError> { self.inner.request("account2.export", rpc_params![address, password]).await.map_err(Into::into) }
    pub async fn change_account_password(self, address: AccountAddress2, new_password: String) -> Result<AccountInfo2, RpcError> { self.inner.request("account2.change_password", rpc_params![address, new_password]).await.map_err(Into::into) }
    pub async fn accepted_tokens(self, address: AccountAddress2) -> Result<Vec<starcoin_vm2_vm_types::account_config::token_code::TokenCode>, RpcError> { self.inner.request("account2.accepted_tokens", rpc_params![address]).await.map_err(Into::into) }
    pub async fn remove(self, address: AccountAddress2, password: Option<String>) -> Result<AccountInfo2, RpcError> { self.inner.request("account2.remove", rpc_params![address, password]).await.map_err(Into::into) }
    pub async fn sign_txn(self, raw_txn: starcoin_vm2_vm_types::transaction::RawUserTransaction, signer: AccountAddress2) -> Result<SignedUserTransaction2, RpcError> { self.inner.request("account2.sign_txn", rpc_params![raw_txn, signer]).await.map_err(Into::into) }
    pub async fn create(self, password: String) -> Result<AccountInfo2, RpcError> { self.inner.request("account2.create", rpc_params![password]).await.map_err(Into::into) }
}

impl StateClient {
    pub async fn get(self, access_path: AccessPath) -> Result<Option<Vec<u8>>, RpcError> { self.inner.request("state.get", rpc_params![access_path]).await.map_err(Into::into) }
    pub async fn get_with_proof(self, access_path: AccessPath) -> Result<StateWithProofView, RpcError> { self.inner.request("state.get_with_proof", rpc_params![access_path]).await.map_err(Into::into) }
    pub async fn get_with_proof_by_root(self, access_path: AccessPath, state_root: HashValue) -> Result<StateWithProofView, RpcError> { self.inner.request("state.get_with_proof_by_root", rpc_params![access_path, state_root]).await.map_err(Into::into) }
    pub async fn get_with_proof_by_root_raw(self, access_path: AccessPath, state_root: HashValue) -> Result<StrView<Vec<u8>>, RpcError> { self.inner.request("state.get_with_proof_by_root_raw", rpc_params![access_path, state_root]).await.map_err(Into::into) }
    pub async fn get_state_root(self) -> Result<HashValue, RpcError> { self.inner.request("state.get_state_root", rpc_params![]).await.map_err(Into::into) }
    pub async fn get_account_state(self, address: AccountAddress) -> Result<Option<AccountState>, RpcError> { self.inner.request("state.get_account_state", rpc_params![address]).await.map_err(Into::into) }
    pub async fn get_account_state_set(self, address: AccountAddress, state_root: Option<HashValue>) -> Result<Option<AccountStateSetView>, RpcError> { self.inner.request("state.get_account_state_set", rpc_params![address, state_root]).await.map_err(Into::into) }
    pub async fn get_resource(self, address: AccountAddress, resource_type: StrView<StructTag>, option: Option<GetResourceOption>) -> Result<Option<ResourceView>, RpcError> { self.inner.request("state.get_resource", rpc_params![address, resource_type, option]).await.map_err(Into::into) }
    pub async fn list_resource(self, address: AccountAddress, option: Option<ListResourceOption>) -> Result<ListResourceView, RpcError> { self.inner.request("state.list_resource", rpc_params![address, option]).await.map_err(Into::into) }
    pub async fn get_code(self, module_id: StrView<ModuleId>, option: Option<GetCodeOption>) -> Result<Option<CodeView>, RpcError> { self.inner.request("state.get_code", rpc_params![module_id, option]).await.map_err(Into::into) }
    pub async fn list_code(self, address: AccountAddress, option: Option<ListCodeOption>) -> Result<ListCodeView, RpcError> { self.inner.request("state.list_code", rpc_params![address, option]).await.map_err(Into::into) }
    pub async fn get_with_table_item_proof_by_root(self, handle: TableHandle, key: Vec<u8>, state_root: HashValue) -> Result<StateWithTableItemProofView, RpcError> { self.inner.request("state.get_with_table_item_proof_by_root", rpc_params![handle, key, state_root]).await.map_err(Into::into) }
    pub async fn get_state_node_by_node_hash(self, key_hash: HashValue) -> Result<Option<Vec<u8>>, RpcError> { self.inner.request("state.get_state_node_by_node_hash", rpc_params![key_hash]).await.map_err(Into::into) }
}

impl StateClient2 {
    pub async fn get(self, state_key: starcoin_vm2_vm_types::state_store::state_key::StateKey) -> Result<Option<Vec<u8>>, RpcError> { self.inner.request("state2.get", rpc_params![state_key]).await.map_err(Into::into) }
    pub async fn get_state_node_by_node_hash(self, key_hash: HashValue) -> Result<Option<Vec<u8>>, RpcError> { self.inner.request("state2.get_state_node_by_node_hash", rpc_params![key_hash]).await.map_err(Into::into) }
    pub async fn get_with_proof(self, state_key: starcoin_vm2_vm_types::state_store::state_key::StateKey) -> Result<starcoin_vm2_types::view::StateWithProofView, RpcError> { self.inner.request("state2.get_with_proof", rpc_params![state_key]).await.map_err(Into::into) }
    pub async fn get_with_proof_raw(self, state_key: starcoin_vm2_vm_types::state_store::state_key::StateKey) -> Result<starcoin_vm2_types::view::StrView<Vec<u8>>, RpcError> { self.inner.request("state2.get_with_proof_raw", rpc_params![state_key]).await.map_err(Into::into) }
    pub async fn get_account_state(self, address: AccountAddress2) -> Result<starcoin_vm2_types::account_state::AccountState, RpcError> { self.inner.request("state2.get_account_state", rpc_params![address]).await.map_err(Into::into) }
    pub async fn get_account_state_set(self, address: AccountAddress2, state_root: Option<HashValue>) -> Result<Option<starcoin_vm2_types::view::AccountStateSetView>, RpcError> { self.inner.request("state2.get_account_state_set", rpc_params![address, state_root]).await.map_err(Into::into) }
    pub async fn get_state_root(self) -> Result<HashValue, RpcError> { self.inner.request("state2.get_state_root", rpc_params![]).await.map_err(Into::into) }
    pub async fn get_with_proof_by_root(self, state_key: starcoin_vm2_vm_types::state_store::state_key::StateKey, state_root: HashValue) -> Result<starcoin_vm2_types::view::StateWithProofView, RpcError> { self.inner.request("state2.get_with_proof_by_root", rpc_params![state_key, state_root]).await.map_err(Into::into) }
    pub async fn get_with_proof_by_root_raw(self, state_key: starcoin_vm2_vm_types::state_store::state_key::StateKey, state_root: HashValue) -> Result<starcoin_vm2_types::view::StrView<Vec<u8>>, RpcError> { self.inner.request("state2.get_with_proof_by_root_raw", rpc_params![state_key, state_root]).await.map_err(Into::into) }
    pub async fn get_with_table_item_proof_by_root(self, handle: starcoin_vm2_vm_types::state_store::table::TableHandle, key: Vec<u8>, state_root: HashValue) -> Result<starcoin_vm2_types::view::StateWithTableItemProofView, RpcError> { self.inner.request("state2.get_with_table_item_proof_by_root", rpc_params![handle, key, state_root]).await.map_err(Into::into) }
    pub async fn get_code(self, module_id: starcoin_vm2_types::view::StrView<starcoin_vm2_vm_types::language_storage::ModuleId>, option: Option<starcoin_vm2_rpc_api::state_api::GetCodeOption>) -> Result<Option<starcoin_vm2_types::view::CodeView>, RpcError> { self.inner.request("state2.get_code", rpc_params![module_id, option]).await.map_err(Into::into) }
    pub async fn get_resource(self, address: AccountAddress2, resource_type: starcoin_vm2_types::view::StrView<starcoin_vm2_vm_types::language_storage::StructTag>, option: Option<starcoin_vm2_rpc_api::state_api::GetResourceOption>) -> Result<Option<starcoin_vm2_types::view::ResourceView>, RpcError> { self.inner.request("state2.get_resource", rpc_params![address, resource_type, option]).await.map_err(Into::into) }
    pub async fn list_resource(self, address: AccountAddress2, option: Option<starcoin_vm2_rpc_api::state_api::ListResourceOption>) -> Result<starcoin_vm2_types::view::ListResourceView, RpcError> { self.inner.request("state2.list_resource", rpc_params![address, option]).await.map_err(Into::into) }
    pub async fn list_code(self, address: AccountAddress2, option: Option<starcoin_vm2_rpc_api::state_api::ListCodeOption>) -> Result<starcoin_vm2_types::view::ListCodeView, RpcError> { self.inner.request("state2.list_code", rpc_params![address, option]).await.map_err(Into::into) }
}

impl ContractClient {
    pub async fn get_code(self, module_id: StrView<ModuleId>) -> Result<Option<StrView<Vec<u8>>>, RpcError> { self.inner.request("contract.get_code", rpc_params![module_id]).await.map_err(Into::into) }
    pub async fn get_resource(self, addr: AccountAddress, resource_type: StrView<StructTag>) -> Result<Option<AnnotatedMoveStructView>, RpcError> { self.inner.request("contract.get_resource", rpc_params![addr, resource_type]).await.map_err(Into::into) }
    pub async fn call_v2(self, call: ContractCall) -> Result<Vec<DecodedMoveValue>, RpcError> { self.inner.request("contract.call_v2", rpc_params![call]).await.map_err(Into::into) }
    pub async fn resolve_function(self, function_id: FunctionIdView) -> Result<FunctionABI, RpcError> { self.inner.request("contract.resolve_function", rpc_params![function_id]).await.map_err(Into::into) }
    pub async fn resolve_struct(self, struct_tag: StructTagView) -> Result<StructInstantiation, RpcError> { self.inner.request("contract.resolve_struct", rpc_params![struct_tag]).await.map_err(Into::into) }
    pub async fn resolve_module(self, module_id: ModuleIdView) -> Result<ModuleABI, RpcError> { self.inner.request("contract.resolve_module", rpc_params![module_id]).await.map_err(Into::into) }
    pub async fn dry_run(self, txn: DryRunTransactionRequest) -> Result<DryRunOutputView, RpcError> { self.inner.request("contract.dry_run", rpc_params![txn]).await.map_err(Into::into) }
    pub async fn dry_run_raw(self, raw_txn: String, sender_public_key: StrView<starcoin_vm_types::transaction::authenticator::AccountPublicKey>) -> Result<DryRunOutputView, RpcError> { self.inner.request("contract.dry_run_raw", rpc_params![raw_txn, sender_public_key]).await.map_err(Into::into) }
}

impl ContractClient2 {
    pub async fn get_code(self, module_id: starcoin_vm2_types::view::StrView<starcoin_vm2_vm_types::language_storage::ModuleId>) -> Result<Option<starcoin_vm2_types::view::StrView<Vec<u8>>>, RpcError> { self.inner.request("contract2.get_code", rpc_params![module_id]).await.map_err(Into::into) }
    pub async fn get_resource(self, addr: AccountAddress2, resource_type: starcoin_vm2_types::view::StrView<starcoin_vm2_vm_types::language_storage::StructTag>) -> Result<Option<starcoin_vm2_types::view::AnnotatedMoveStructView>, RpcError> { self.inner.request("contract2.get_resource", rpc_params![addr, resource_type]).await.map_err(Into::into) }
    pub async fn call_v2(self, call: starcoin_vm2_types::view::ContractCall) -> Result<Vec<starcoin_vm2_rpc_api::DecodedMoveValue>, RpcError> { self.inner.request("contract2.call_v2", rpc_params![call]).await.map_err(Into::into) }
    pub async fn resolve_function(self, function_id: starcoin_vm2_types::view::FunctionIdView) -> Result<starcoin_vm2_abi_types::FunctionABI, RpcError> { self.inner.request("contract2.resolve_function", rpc_params![function_id]).await.map_err(Into::into) }
    pub async fn resolve_struct(self, struct_tag: starcoin_vm2_types::view::StructTagView) -> Result<starcoin_vm2_abi_types::StructInstantiation, RpcError> { self.inner.request("contract2.resolve_struct", rpc_params![struct_tag]).await.map_err(Into::into) }
    pub async fn resolve_module(self, module_id: starcoin_vm2_types::view::ModuleIdView) -> Result<starcoin_vm2_abi_types::ModuleABI, RpcError> { self.inner.request("contract2.resolve_module", rpc_params![module_id]).await.map_err(Into::into) }
    pub async fn dry_run(self, txn: starcoin_vm2_types::view::DryRunTransactionRequest) -> Result<starcoin_vm2_types::view::DryRunOutputView, RpcError> { self.inner.request("contract2.dry_run", rpc_params![txn]).await.map_err(Into::into) }
    pub async fn dry_run_raw(self, raw_txn: String, sender_public_key: starcoin_vm2_types::view::StrView<starcoin_vm2_vm_types::transaction::authenticator::AccountPublicKey>) -> Result<starcoin_vm2_types::view::DryRunOutputView, RpcError> { self.inner.request("contract2.dry_run_raw", rpc_params![raw_txn, sender_public_key]).await.map_err(Into::into) }
}

impl DebugClient {
    pub async fn set_log_level(self, logger_name: Option<String>, level: String) -> Result<(), RpcError> { self.inner.request("debug.set_log_level", rpc_params![logger_name, level]).await.map_err(Into::into) }
    pub async fn set_log_pattern(self, pattern: starcoin_logger::LogPattern) -> Result<(), RpcError> { self.inner.request("debug.set_log_pattern", rpc_params![pattern]).await.map_err(Into::into) }
    pub async fn panic(self) -> Result<(), RpcError> { self.inner.request("debug.panic", rpc_params![]).await.map_err(Into::into) }
    pub async fn txfactory_status(self, action: FactoryAction) -> Result<bool, RpcError> { self.inner.request("txfactory.status", rpc_params![action]).await.map_err(Into::into) }
    pub async fn sleep(self, time: u64) -> Result<(), RpcError> { self.inner.request("debug.sleep", rpc_params![time]).await.map_err(Into::into) }
    pub async fn set_concurrency_level(self, level: usize) -> Result<(), RpcError> { self.inner.request("debug.set_concurrency_level", rpc_params![level]).await.map_err(Into::into) }
    pub async fn get_concurrency_level(self) -> Result<usize, RpcError> { self.inner.request("debug.get_concurrency_level", rpc_params![]).await.map_err(Into::into) }
    pub async fn set_logger_balance_amount(self, balance_amount: u64) -> Result<(), RpcError> { self.inner.request("debug.set_logger_balance_amount", rpc_params![balance_amount]).await.map_err(Into::into) }
    pub async fn get_logger_balance_amount(self) -> Result<u64, RpcError> { self.inner.request("debug.get_logger_balance_amount", rpc_params![]).await.map_err(Into::into) }
}

impl ChainClient {
    pub async fn id(self) -> Result<ChainId, RpcError> { self.inner.request("chain.id", rpc_params![]).await.map_err(Into::into) }
    pub async fn info(self) -> Result<ChainInfoView, RpcError> { self.inner.request("chain.info", rpc_params![]).await.map_err(Into::into) }
    pub async fn get_headers(self, block_hashes: Vec<HashValue>) -> Result<Vec<BlockHeaderView>, RpcError> { self.inner.request("chain.get_headers", rpc_params![block_hashes]).await.map_err(Into::into) }
    pub async fn get_block_by_hash(self, hash: HashValue, option: Option<GetBlockOption>) -> Result<Option<BlockView>, RpcError> { self.inner.request("chain.get_block_by_hash", rpc_params![hash, option]).await.map_err(Into::into) }
    pub async fn get_block_by_number(self, number: BlockNumber, option: Option<GetBlockOption>) -> Result<Option<BlockView>, RpcError> { self.inner.request("chain.get_block_by_number", rpc_params![number, option]).await.map_err(Into::into) }
    pub async fn get_block_info_by_number(self, number: BlockNumber) -> Result<Option<BlockInfoView>, RpcError> { self.inner.request("chain.get_block_info_by_number", rpc_params![number]).await.map_err(Into::into) }
    pub async fn get_block_info_by_number2(self, number: BlockNumber) -> Result<Option<BlockInfoView2>, RpcError> { self.inner.request("chain.get_block_info_by_number2", rpc_params![number]).await.map_err(Into::into) }
    pub async fn get_block_info_by_hash(self, id: HashValue) -> Result<Option<BlockInfoView>, RpcError> { self.inner.request("chain.get_block_info_by_hash", rpc_params![id]).await.map_err(Into::into) }
    pub async fn get_blocks_by_number(self, number: Option<BlockNumber>, count: u64, option: Option<GetBlocksOption>) -> Result<Vec<BlockView>, RpcError> { self.inner.request("chain.get_blocks_by_number", rpc_params![number, count, option]).await.map_err(Into::into) }
    pub async fn get_transaction(self, txn_id: HashValue, option: Option<GetTransactionOption>) -> Result<Option<TransactionView>, RpcError> { self.inner.request("chain.get_transaction", rpc_params![txn_id, option]).await.map_err(Into::into) }
    pub async fn get_transaction2(self, txn_id: HashValue, option: Option<GetTransactionOption>) -> Result<Option<TransactionView2>, RpcError> { self.inner.request("chain.get_transaction2", rpc_params![txn_id, option]).await.map_err(Into::into) }
    pub async fn get_transaction_info(self, txn_hash: HashValue) -> Result<Option<TransactionInfoView>, RpcError> { self.inner.request("chain.get_transaction_info", rpc_params![txn_hash]).await.map_err(Into::into) }
    pub async fn get_transaction_info2(self, txn_hash: HashValue) -> Result<Option<TransactionInfoView2>, RpcError> { self.inner.request("chain.get_transaction_info2", rpc_params![txn_hash]).await.map_err(Into::into) }
    pub async fn get_events_by_txn_hash(self, txn_hash: HashValue, option: Option<GetEventOption>) -> Result<Vec<TransactionEventResponse>, RpcError> { self.inner.request("chain.get_events_by_txn_hash", rpc_params![txn_hash, option]).await.map_err(Into::into) }
    pub async fn get_events_by_txn_hash2(self, txn_hash: HashValue, option: Option<GetEventOption>) -> Result<Vec<starcoin_vm2_types::view::TransactionEventResponse>, RpcError> { self.inner.request("chain.get_events_by_txn_hash2", rpc_params![txn_hash, option]).await.map_err(Into::into) }
    pub async fn get_block_txn_infos(self, block_id: HashValue) -> Result<Vec<TransactionInfoView>, RpcError> { self.inner.request("chain.get_block_txn_infos", rpc_params![block_id]).await.map_err(Into::into) }
    pub async fn get_block_txn_infos2(self, block_id: HashValue) -> Result<Vec<TransactionInfoView2>, RpcError> { self.inner.request("chain.get_block_txn_infos2", rpc_params![block_id]).await.map_err(Into::into) }
    pub async fn get_block_txn_infos_in_seq(self, block_id: HashValue) -> Result<Vec<TransactionInfoViewEnum>, RpcError> { self.inner.request("chain.get_block_txn_infos_in_seq", rpc_params![block_id]).await.map_err(Into::into) }
    pub async fn get_txn_info_by_block_and_index(self, block_id: HashValue, idx: u64) -> Result<Option<TransactionInfoView>, RpcError> { self.inner.request("chain.get_txn_info_by_block_and_index", rpc_params![block_id, idx]).await.map_err(Into::into) }
    pub async fn get_txn_info_by_block_and_index2(self, block_id: HashValue, idx: u64) -> Result<Option<TransactionInfoView2>, RpcError> { self.inner.request("chain.get_txn_info_by_block_and_index2", rpc_params![block_id, idx]).await.map_err(Into::into) }
    pub async fn get_transaction_infos(self, start_global_index: u64, reverse: bool, max_size: u64) -> Result<Vec<TransactionInfoView>, RpcError> { self.inner.request("chain.get_transaction_infos", rpc_params![start_global_index, reverse, max_size]).await.map_err(Into::into) }
    pub async fn get_transaction_infos2(self, start_global_index: u64, reverse: bool, max_size: u64) -> Result<Vec<TransactionInfoView2>, RpcError> { self.inner.request("chain.get_transaction_infos2", rpc_params![start_global_index, reverse, max_size]).await.map_err(Into::into) }
    pub async fn get_transaction_proof(self, block_hash: HashValue, transaction_global_index: u64, event_index: Option<u64>, access_path: Option<starcoin_types::access_path::AccessPath>) -> Result<Option<TransactionInfoWithProofView>, RpcError> { self.inner.request("chain.get_transaction_proof", rpc_params![block_hash, transaction_global_index, event_index, access_path]).await.map_err(Into::into) }
    pub async fn get_transaction_proof_raw(self, block_hash: HashValue, transaction_global_index: u64, event_index: Option<u64>, access_path: Option<starcoin_types::access_path::AccessPath>) -> Result<Option<StrView<Vec<u8>>>, RpcError> { self.inner.request("chain.get_transaction_proof_raw", rpc_params![block_hash, transaction_global_index, event_index, access_path]).await.map_err(Into::into) }
    pub async fn get_transaction_proof2(self, block_hash: HashValue, transaction_global_index: u64, event_index: Option<u64>, access_path: Option<MultiAccessPath>) -> Result<Option<TransactionInfoWithProofView>, RpcError> { self.inner.request("chain.get_transaction_proof2", rpc_params![block_hash, transaction_global_index, event_index, access_path]).await.map_err(Into::into) }
    pub async fn get_transaction_proof2_raw(self, block_hash: HashValue, transaction_global_index: u64, event_index: Option<u64>, access_path: Option<MultiAccessPath>) -> Result<Option<starcoin_vm2_types::view::StrView<Vec<u8>>>, RpcError> { self.inner.request("chain.get_transaction_proof2_raw", rpc_params![block_hash, transaction_global_index, event_index, access_path]).await.map_err(Into::into) }
    pub async fn get_vm_multi_state(self, block_hash: HashValue) -> Result<Option<MultiStateView>, RpcError> { self.inner.request("chain.get_vm_multi_state", rpc_params![block_hash]).await.map_err(Into::into) }
}

impl MinerClient {
    pub async fn submit(self, minting_blob: String, nonce: u32, extra: String) -> Result<MintedBlockView, RpcError> {
        self.inner.request("mining.submit", rpc_params![minting_blob, nonce, extra]).await.map_err(Into::into)
    }
}

impl SyncManagerClient {
    pub async fn status(self) -> Result<SyncStatusView, RpcError> { self.inner.request("sync.status", rpc_params![]).await.map_err(Into::into) }
    pub async fn progress(self) -> Result<Option<SyncProgressReport>, RpcError> { self.inner.request("sync.progress", rpc_params![]).await.map_err(Into::into) }
    pub async fn peer_score(self) -> Result<PeerScoreResponse, RpcError> { self.inner.request("sync.score", rpc_params![]).await.map_err(Into::into) }
    pub async fn start(self, force: bool, peers: Vec<PeerId>, skip_pow_verify: bool, strategy: Option<PeerStrategy>) -> Result<(), RpcError> { self.inner.request("sync.start", rpc_params![force, peers, skip_pow_verify, strategy]).await.map_err(Into::into) }
    pub async fn cancel(self) -> Result<(), RpcError> { self.inner.request("sync.cancel", rpc_params![]).await.map_err(Into::into) }
}

impl NetworkManagerClient {
    pub async fn known_peers(self) -> Result<Vec<PeerId>, RpcError> { self.inner.request("network_manager.known_peers", rpc_params![]).await.map_err(Into::into) }
    pub async fn state(self) -> Result<NetworkState, RpcError> { self.inner.request("network_manager.state", rpc_params![]).await.map_err(Into::into) }
    pub async fn get_address(self, peer_id: String) -> Result<Vec<Multiaddr>, RpcError> { self.inner.request("network_manager.get_address", rpc_params![peer_id]).await.map_err(Into::into) }
    pub async fn add_peer(self, peer: String) -> Result<(), RpcError> { self.inner.request("network_manager.add_peer", rpc_params![peer]).await.map_err(Into::into) }
    pub async fn call_peer(self, peer_id: String, rpc_method: String, message: StrView<Vec<u8>>) -> Result<StrView<Vec<u8>>, RpcError> { self.inner.request("network_manager.call", rpc_params![peer_id, rpc_method, message]).await.map_err(Into::into) }
    pub async fn set_peer_reputation(self, peer_id: String, reput: i32) -> Result<(), RpcError> { self.inner.request("network_manager.set_peer_reput", rpc_params![peer_id, reput]).await.map_err(Into::into) }
    pub async fn ban_peer(self, peer_id: String, ban: bool) -> Result<(), RpcError> { self.inner.request("network_manager.ban_peer", rpc_params![peer_id, ban]).await.map_err(Into::into) }
}

impl PubSubClient {
    pub async fn subscribe_events(self,
        filter: EventFilter,
        decode: bool,
    ) -> Result<Subscription<TransactionEventView>, RpcError> {
        self.inner
            .subscribe(
                "starcoin_subscribe",
                rpc_params![Kind::Events, EventParams { filter, decode }],
                "starcoin_unsubscribe",
            )
            .await
            .map_err(Into::into)
    }

    pub async fn subscribe_events_v2(self,
        filter: EventFilterV2,
        decode: bool,
    ) -> Result<Subscription<starcoin_vm2_types::view::TransactionEventView>, RpcError> {
        self.inner
            .subscribe(
                "starcoin_subscribe",
                rpc_params![Kind::Events, EventParamsV2::new(filter, decode)],
                "starcoin_unsubscribe",
            )
            .await
            .map_err(Into::into)
    }

    pub async fn subscribe_new_block(self) -> Result<Subscription<BlockView>, RpcError> {
        self.inner
            .subscribe(
                "starcoin_subscribe",
                rpc_params![vec![Kind::NewHeads]],
                "starcoin_unsubscribe",
            )
            .await
            .map_err(Into::into)
    }

    pub async fn subscribe_new_transactions(self,
    ) -> Result<Subscription<Vec<HashValue>>, RpcError> {
        self.inner
            .subscribe(
                "starcoin_subscribe",
                rpc_params![vec![Kind::NewPendingTransactions]],
                "starcoin_unsubscribe",
            )
            .await
            .map_err(Into::into)
    }

    pub async fn subscribe_new_mint_block(self) -> Result<Subscription<MintBlockEvent>, RpcError> {
        self.inner
            .subscribe(
                "starcoin_subscribe",
                rpc_params![vec![Kind::NewMintBlock]],
                "starcoin_unsubscribe",
            )
            .await
            .map_err(Into::into)
    }
}
