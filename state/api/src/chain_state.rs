// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::StateReaderExt;
use crate::TABLE_PATH_LIST;
use anyhow::{ensure, Result};
use forkable_jellyfish_merkle::{blob::Blob, proof::SparseMerkleProof, RawKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_types::language_storage::StructTag;
use starcoin_types::state_set::AccountStateSet;
use starcoin_types::write_set::WriteSet;
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_config::AccountResource,
    account_state::AccountState, state_set::ChainStateSet,
};
use starcoin_vm_types::account_config::TABLE_HANDLE_ADDRESS_LIST;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::on_chain_config::FlexiDagConfigV2;
use starcoin_vm_types::on_chain_resource::{Epoch, EpochInfo, GlobalTimeOnChain};
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::{
    move_resource::MoveResource, on_chain_config::OnChainConfig, state_view::StateView,
};
use std::convert::TryFrom;

#[derive(Debug, Default, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct StateProof {
    pub account_state: Option<Blob>,
    pub account_proof: SparseMerkleProof,
    pub account_state_proof: SparseMerkleProof,
}

impl StateProof {
    pub fn new(
        account_state: Option<Vec<u8>>,
        account_proof: SparseMerkleProof,
        account_state_proof: SparseMerkleProof,
    ) -> Self {
        Self {
            account_state: account_state.map(Blob::from),
            account_proof,
            account_state_proof,
        }
    }
    /// verify the resource blob with `access_path`,
    /// given expected_root_hash, and expected account state blob.
    pub fn verify(
        &self,
        expected_root_hash: HashValue,
        access_path: AccessPath,
        access_resource_blob: Option<&[u8]>,
    ) -> Result<()> {
        let (account_address, data_path) = access_path.into_inner();
        match self.account_state.as_ref() {
            None => {
                ensure!(
                    access_resource_blob.is_none(),
                    "accessed resource should not exists"
                );
            }
            Some(s) => {
                let account_state = AccountState::try_from(s.as_ref())?;
                match account_state.storage_roots()[data_path.data_type().storage_index()] {
                    None => {
                        ensure!(
                            access_resource_blob.is_none(),
                            "accessed resource should not exists"
                        );
                    }
                    Some(expected_hash) => {
                        let blob = access_resource_blob.map(|data| Blob::from(data.to_vec()));
                        self.account_state_proof.verify(
                            expected_hash,
                            data_path.key_hash(),
                            blob.as_ref(),
                        )?;
                    }
                }
            }
        }
        self.account_proof.verify(
            expected_root_hash,
            account_address.key_hash(),
            self.account_state.as_ref(),
        )
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct StateWithProof {
    pub state: Option<Vec<u8>>,
    pub proof: StateProof,
}

impl StateWithProof {
    pub fn new(state: Option<Vec<u8>>, proof: StateProof) -> Self {
        Self { state, proof }
    }

    pub fn get_state(&self) -> &Option<Vec<u8>> {
        &self.state
    }

    pub fn verify(&self, expect_root: HashValue, access_path: AccessPath) -> Result<()> {
        self.proof
            .verify(expect_root, access_path, self.state.as_deref())
    }
}

pub trait ChainStateReader: StateView {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof>;

    /// Gets account state
    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>>;

    /// get whole state data of some account address.
    fn get_account_state_set(&self, address: &AccountAddress) -> Result<Option<AccountStateSet>>;

    fn exist_account(&self, address: &AccountAddress) -> Result<bool> {
        self.get_account_state(address).map(|state| state.is_some())
    }

    /// Gets current state root.
    fn state_root(&self) -> HashValue;

    fn dump(&self) -> Result<ChainStateSet>;

    fn dump_iter(&self) -> Result<AccountStateSetIterator>;

    fn get_with_table_item_proof(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> Result<StateWithTableItemProof>;

    fn get_table_info(&self, address: AccountAddress) -> Result<Option<TableInfo>>;
}

pub trait ChainStateWriter {
    /// Sets state at access_path.
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()>;

    /// Remove state at access_path
    fn remove(&self, access_path: &AccessPath) -> Result<()>;
    /// Apply dump result to ChainState
    fn apply(&self, state_set: ChainStateSet) -> Result<()>;

    //TODO support batch write.
    fn apply_write_set(&self, write_set: WriteSet) -> Result<()>;

    fn commit(&self) -> Result<HashValue>;

    fn flush(&self) -> Result<()>;
}
/// `AccountStateReader` is a helper struct for read account state.
pub struct AccountStateReader<'a, Reader> {
    //TODO add a cache.
    reader: &'a Reader,
}

impl<'a, Reader> AccountStateReader<'a, Reader>
where
    Reader: ChainStateReader,
{
    pub fn new(reader: &'a Reader) -> Self {
        Self { reader }
    }

    /// Get AccountResource by address
    pub fn get_account_resource(
        &self,
        address: &AccountAddress,
    ) -> Result<Option<AccountResource>> {
        self.reader.get_account_resource(*address)
    }

    /// Get Resource by type
    pub fn get_resource<R>(&self, address: AccountAddress) -> Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        self.reader.get_resource(address)
    }

    pub fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        self.reader.get_sequence_number(address)
    }

    pub fn get_on_chain_config<C>(&self) -> Result<Option<C>>
    where
        C: OnChainConfig,
    {
        self.reader.get_on_chain_config()
    }

    pub fn get_balance(&self, address: &AccountAddress) -> Result<Option<u128>> {
        self.reader.get_balance(*address)
    }

    pub fn get_dag_config(&self) -> Result<Option<FlexiDagConfigV2>> {
        self.get_on_chain_config::<FlexiDagConfigV2>()
    }

    /// Get balance by address and coin type
    pub fn get_balance_by_type(
        &self,
        address: &AccountAddress,
        type_tag: StructTag,
    ) -> Result<Option<u128>> {
        self.reader.get_balance_by_type(*address, type_tag)
    }

    pub fn get_balance_by_token_code(
        &self,
        address: &AccountAddress,
        token_code: TokenCode,
    ) -> Result<Option<u128>> {
        self.reader.get_balance_by_token_code(*address, token_code)
    }

    pub fn get_epoch(&self) -> Result<Epoch> {
        self.reader.get_epoch()
    }

    pub fn get_epoch_info(&self) -> Result<EpochInfo> {
        self.reader.get_epoch_info()
    }

    pub fn get_timestamp(&self) -> Result<GlobalTimeOnChain> {
        self.reader.get_timestamp()
    }

    pub fn get_chain_id(&self) -> Result<ChainId> {
        self.reader.get_chain_id()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct StateWithTableItemProof {
    pub state_proof: (StateWithProof, HashValue),
    pub table_handle_proof: (Option<Vec<u8>>, SparseMerkleProof, HashValue),
    pub key_proof: (Option<Vec<u8>>, SparseMerkleProof, HashValue),
}

impl StateWithTableItemProof {
    pub fn new(
        state_proof: (StateWithProof, HashValue),
        table_handle_proof: (Option<Vec<u8>>, SparseMerkleProof, HashValue),
        key_proof: (Option<Vec<u8>>, SparseMerkleProof, HashValue),
    ) -> Self {
        Self {
            state_proof,
            table_handle_proof,
            key_proof,
        }
    }

    pub fn verify(&self, handle: &TableHandle, key: &[u8]) -> Result<()> {
        let idx = handle.get_idx()?;
        let handle_address = TABLE_HANDLE_ADDRESS_LIST
            .get(idx)
            .expect("get TABLE_HANDLE_ADDRESS_LIST should always succeed");
        let table_path = TABLE_PATH_LIST
            .get(idx)
            .expect("get TABLE_PATH_LIST should always succeed");
        self.state_proof.0.proof.verify(
            self.state_proof.1,
            AccessPath::new(*handle_address, table_path.clone()),
            self.state_proof.0.state.as_deref(),
        )?;
        self.table_handle_proof.1.verify(
            self.table_handle_proof.2,
            handle.key_hash(),
            self.table_handle_proof
                .0
                .as_ref()
                .map(|v| Blob::from(v.clone()))
                .as_ref(),
        )?;
        self.key_proof.1.verify(
            self.key_proof.2,
            key.to_vec().key_hash(),
            self.key_proof
                .0
                .as_ref()
                .map(|v| Blob::from(v.clone()))
                .as_ref(),
        )?;
        Ok(())
    }
}
