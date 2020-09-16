// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for running the VM to execute and verify transactions.
use crate::account::{Account, AccountData};
use anyhow::Result;
use starcoin_config::ChainNetwork;
use starcoin_crypto::HashValue;
use starcoin_genesis::Genesis;
use starcoin_statedb::{ChainStateDB, ChainStateWriter};
use starcoin_types::write_set::{WriteOp, WriteSetMut};
use starcoin_types::{
    access_path::AccessPath,
    block_metadata::BlockMetadata,
    transaction::{SignedUserTransaction, Transaction, TransactionOutput},
    write_set::WriteSet,
};
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm_types::account_config::STC_TOKEN_CODE_STR;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::{
    account_config::{association_address, AccountResource, BalanceResource},
    file_format::CompiledModule,
    language_storage::ModuleId,
    state_view::StateView,
    vm_status::VMStatus,
};

/// Provides an environment to run a VM instance.
pub struct FakeExecutor {
    data_store: ChainStateDB,
    block_time: u64,
    chain_id: ChainId,
}

impl Default for FakeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeExecutor {
    pub fn new() -> Self {
        let net = &ChainNetwork::TEST;
        let genesis_txn = Genesis::build_genesis_transaction(net).unwrap();
        let data_store = ChainStateDB::mock();
        Genesis::execute_genesis_txn(&data_store, genesis_txn).unwrap();
        Self {
            data_store,
            block_time: 0,
            chain_id: net.chain_id(),
        }
    }

    /// Creates an executor from a genesis [`WriteSet`].
    pub fn from_genesis(write_set: &WriteSet) -> Self {
        let mut executor = FakeExecutor {
            data_store: ChainStateDB::mock(),
            block_time: 0,
            chain_id: ChainNetwork::TEST.chain_id(),
        };
        executor.apply_write_set(write_set);
        executor
    }

    /// Creates an executor in which no genesis state has been applied yet.
    pub fn no_genesis() -> Self {
        FakeExecutor {
            data_store: ChainStateDB::mock(),
            block_time: 0,
            chain_id: ChainNetwork::TEST.chain_id(),
        }
    }

    /// Creates a number of [`Account`] instances all with the same balance and sequence number,
    /// and publishes them to this executor's data store.
    pub fn create_accounts(&mut self, size: usize, balance: u128, seq_num: u64) -> Vec<Account> {
        let mut accounts: Vec<Account> = Vec::with_capacity(size);
        for _i in 0..size {
            let account_data = AccountData::new(balance, seq_num);
            self.add_account_data(&account_data);
            accounts.push(account_data.into_account());
        }
        accounts
    }

    /// Applies a [`WriteSet`] to this executor's data store.
    pub fn apply_write_set(&mut self, write_set: &WriteSet) {
        self.data_store
            .apply_write_set(write_set.clone())
            .expect("statedb apply write set should work.");
    }

    /// Adds an account to this executor's data store.
    pub fn add_account_data(&mut self, account_data: &AccountData) {
        let write_set = account_data.to_writeset();
        self.apply_write_set(&write_set)
    }

    /// Adds a module to this executor's data store.
    ///
    /// Does not do any sort of verification on the module.
    pub fn add_module(&mut self, module_id: &ModuleId, module: &CompiledModule) {
        let access_path = AccessPath::from(module_id);
        let mut blob = vec![];
        module
            .serialize(&mut blob)
            .expect("serializing this module should work");
        self.data_store
            .apply_write_set(
                WriteSetMut::new(vec![(access_path, WriteOp::Value(blob))])
                    .freeze()
                    .expect("freeze write_set must success."),
            )
            .expect("statedb set should success");
    }

    /// Reads the resource [`Value`] for an account from this executor's data store.
    pub fn read_account_resource(&self, account: &Account) -> Option<AccountResource> {
        let ap = account.make_account_access_path();
        let data_blob = self
            .data_store
            .get(&ap)
            .expect("account must exist in data store")
            .expect("data must exist in data store");
        scs::from_bytes(data_blob.as_slice()).ok()
    }

    /// Reads the balance resource value for an account from this executor's data store.
    pub fn read_balance_resource(&self, account: &Account) -> Option<BalanceResource> {
        Some(self.read_account_info(account)?.1)
    }

    // Reads the balance resource value for an account from this executor's data store with the
    // given balance currency_code.
    fn read_balance_resource_from_token_code(
        &self,
        account: &Account,
        balance_token_code: &str,
    ) -> Option<BalanceResource> {
        let ap = account.make_balance_access_path(balance_token_code);
        let data_blob = self
            .data_store
            .get(&ap)
            .expect("account must exist in data store")
            .expect("data must exist in data store");
        scs::from_bytes(data_blob.as_slice()).ok()
    }

    /// Reads the AccountResource and BalanceResource for this account. These are coupled together.
    pub fn read_account_info(
        &self,
        account: &Account,
    ) -> Option<(AccountResource, BalanceResource)> {
        self.read_account_resource(account).and_then(|ar| {
            self.read_balance_resource_from_token_code(account, STC_TOKEN_CODE_STR)
                .map(|br| (ar, br))
        })
    }

    /// Executes the given block of transactions.
    ///
    /// Typical tests will call this method and check that the output matches what was expected.
    /// However, this doesn't apply the results of successful transactions to the data store.
    pub fn execute_block(
        &self,
        txn_block: Vec<SignedUserTransaction>,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>> {
        self.execute_transaction_block(
            txn_block
                .iter()
                .map(|txn| Transaction::UserTransaction(txn.clone()))
                .collect(),
        )
    }

    pub fn execute_transaction_block(
        &self,
        txn_block: Vec<Transaction>,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>> {
        let mut vm = StarcoinVM::new();
        vm.execute_transactions(&self.data_store, txn_block)
    }

    pub fn execute_transaction(&self, txn: SignedUserTransaction) -> (VMStatus, TransactionOutput) {
        let txn_block = vec![txn];
        let mut outputs = self
            .execute_block(txn_block)
            .expect("The VM should not fail to startup");
        outputs
            .pop()
            .expect("A block with one transaction should have one output")
    }

    /// Get the blob for the associated AccessPath
    pub fn read_from_access_path(&self, path: &AccessPath) -> Option<Vec<u8>> {
        StateView::get(&self.data_store, path).unwrap()
    }

    /// Verifies the given transaction by running it through the VM verifier.
    pub fn verify_transaction(&self, txn: SignedUserTransaction) -> Option<VMStatus> {
        let mut vm = StarcoinVM::new();
        vm.verify_transaction(&self.data_store, txn)
    }

    pub fn get_state_view(&self) -> &ChainStateDB {
        &self.data_store
    }

    pub fn new_block(&mut self) {
        //TODO refactor block time.
        self.block_time += 1;
        let new_block = BlockMetadata::new(
            HashValue::zero(),
            0,
            association_address(),
            None,
            0,
            self.block_time,
            self.chain_id,
        );
        let (_vm_status, output) = self
            .execute_transaction_block(vec![Transaction::BlockMetadata(new_block)])
            .expect("Executing block prologue should succeed")
            .pop()
            .expect("Failed to get the execution result for Block Prologue");
        // check if we emit the expected event, there might be more events for transaction fees
        //let event = output.events()[0].clone();
        //TODO block event.
        //assert!(event.key() == &new_block_event_key());
        //assert!(scs::from_bytes::<NewBlockEvent>(event.event_data()).is_ok());
        self.apply_write_set(output.write_set());
    }
}

#[cfg(test)]
mod tests {
    use crate::account::AccountData;
    use crate::executor::FakeExecutor;

    #[test]
    fn test_executor() {
        let account_data = AccountData::new(10_000_000, 0);
        let mut executor = FakeExecutor::no_genesis();
        executor.add_account_data(&account_data);
        let resource = executor.read_account_resource(account_data.account());
        assert!(resource.is_some());
    }
}
