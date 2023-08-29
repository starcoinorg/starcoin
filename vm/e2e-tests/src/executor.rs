// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

//! Support for running the VM to execute and verify transactions.

use crate::account::{Account, AccountData};
use crate::golden_outputs::GoldenOutputs;
use move_core_types::vm_status::KeptVMStatus;
use move_table_extension::NativeTableContext;
use num_cpus;
use serde::{Deserialize, Serialize};
use starcoin_config::ChainNetwork;
use starcoin_crypto::keygen::KeyGen;
use starcoin_crypto::HashValue;
use starcoin_gas::{StarcoinGasMeter, StarcoinGasParameters};
use starcoin_gas_algebra_ext::InitialGasSchedule;
use starcoin_vm_runtime::data_cache::{AsMoveResolver, RemoteStorage};
use starcoin_vm_runtime::move_vm_ext::{MoveVmExt, SessionId, SessionOutput};
use starcoin_vm_runtime::parallel_executor::ParallelStarcoinVM;
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm_runtime::VMExecutor;
use starcoin_vm_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::block::NewBlockEvent,
    account_config::{AccountResource, BalanceResource, CORE_CODE_ADDRESS},
    block_metadata::BlockMetadata,
    errors::Location,
    genesis_config::ChainId,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    move_resource::MoveResource,
    on_chain_config::{OnChainConfig, VMConfig, Version},
    state_store::state_key::StateKey,
    state_view::StateView,
    transaction::authenticator::AuthenticationKey,
    transaction::{SignedUserTransaction, Transaction, TransactionOutput, TransactionStatus},
    vm_status::VMStatus,
    write_set::WriteSet,
};

use crate::data_store::FakeDataStore;
use starcoin_statedb::ChainStateWriter;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};
use test_helper::Genesis;

static RNG_SEED: [u8; 32] = [9u8; 32];

const ENV_TRACE_DIR: &str = "TRACE";

/// Directory structure of the trace dir
pub const TRACE_FILE_NAME: &str = "name";
pub const TRACE_FILE_ERROR: &str = "error";
pub const TRACE_DIR_META: &str = "meta";
pub const TRACE_DIR_DATA: &str = "data";
pub const TRACE_DIR_INPUT: &str = "input";
pub const TRACE_DIR_OUTPUT: &str = "output";

/// Maps block number N to the index of the input and output transactions
pub type TraceSeqMapping = (usize, Vec<usize>, Vec<usize>);

/// Provides an environment to run a VM instance.
///
/// This struct is a mock in-memory implementation of the Aptos executor.
#[derive(Debug)]
pub struct FakeExecutor {
    data_store: FakeDataStore,
    block_time: u64,
    executed_output: Option<GoldenOutputs>,
    trace_dir: Option<PathBuf>,
    rng: KeyGen,
}

impl FakeExecutor {
    /// Creates an executor from a genesis [`WriteSet`].
    // pub fn from_genesis(write_set: &WriteSet) -> Self {
    //     let mut executor = Self::no_genesis();
    //     executor.apply_write_set(write_set);
    //     executor
    // }

    /// Create an executor from a saved genesis blob
    // TODO(BobOng): e2e-test
    // pub fn from_saved_genesis(saved_genesis_blob: &[u8]) -> Self {
    //     let change_set = bcs::from_bytes::<ChangeSet>(saved_genesis_blob).unwrap();
    //     Self::from_genesis(change_set.write_set())
    // }

    /// Creates an executor from the genesis file GENESIS_FILE_LOCATION
    pub fn from_genesis_file() -> Self {
        // TODO(BobOng): e2e-test
        //Self::from_genesis(GENESIS_CHANGE_SET.clone().write_set())
        Self::from_test_genesis()
    }

    /// Creates an executor using the standard genesis.
    pub fn from_fresh_genesis() -> Self {
        // TODO(BobOng): e2e-test
        //Self::from_genesis(GENESIS_CHANGE_SET_FRESH.clone().write_set())
        Self::no_genesis()
    }

    pub fn allowlist_genesis() -> Self {
        // Self::custom_genesis(
        //     cached_framework_packages::module_blobs(),
        //     None,
        //     VMPublishingOption::open(),
        // )
        // TODO(BobOng): e2e-test
        Self::no_genesis()
    }

    pub fn from_test_genesis() -> Self {
        //let (state_db, net) = prepare_genesis();
        let fake_executor = Self::no_genesis();
        let net = ChainNetwork::new_test();
        let genesis_txn = Genesis::build_genesis_transaction(&net).unwrap();
        let _txn_info =
            Genesis::execute_genesis_txn(fake_executor.get_state_view(), genesis_txn).unwrap();
        fake_executor
    }

    /// Creates an executor from the genesis file GENESIS_FILE_LOCATION with script/module
    /// publishing options given by `publishing_options`. These can only be either `Open` or
    /// `CustomScript`.
    pub fn from_genesis_with_options(_publishing_options: VMConfig) -> Self {
        // if !publishing_options.is_open_script() {
        //     panic!("Allowlisted transactions are not supported as a publishing option")
        // }

        // Self::custom_genesis(
        //     cached_framework_packages::module_blobs(),
        //     None,
        //     publishing_options,
        // )
        // TODO(BobOng): e2e-test
        Self::no_genesis()
    }

    /// Creates an executor in which no genesis state has been applied yet.
    pub fn no_genesis() -> Self {
        FakeExecutor {
            data_store: FakeDataStore::default(),
            block_time: 0,
            executed_output: None,
            trace_dir: None,
            rng: KeyGen::from_seed(RNG_SEED),
        }
    }

    pub fn set_golden_file(&mut self, test_name: &str) {
        // 'test_name' includes ':' in the names, lets re-write these to be '_'s so that these
        // files can persist on windows machines.
        let file_name = test_name.replace(':', "_");
        self.executed_output = Some(GoldenOutputs::new(&file_name));

        // NOTE: tracing is only available when
        //  - the e2e test outputs a golden file, and
        //  - the environment variable is properly set
        if let Some(env_trace_dir) = env::var_os(ENV_TRACE_DIR) {
            let starcoin_version = Version::fetch_config(&self.data_store.as_move_resolver())
                .map_or(0, |v| v.unwrap().major);

            let trace_dir = Path::new(&env_trace_dir).join(file_name);
            if trace_dir.exists() {
                fs::remove_dir_all(&trace_dir).expect("Failed to clean up the trace directory");
            }
            fs::create_dir_all(&trace_dir).expect("Failed to create the trace directory");
            let mut name_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(trace_dir.join(TRACE_FILE_NAME))
                .unwrap();
            write!(name_file, "{}::{}", test_name, starcoin_version).unwrap();
            for sub_dir in &[
                TRACE_DIR_META,
                TRACE_DIR_DATA,
                TRACE_DIR_INPUT,
                TRACE_DIR_OUTPUT,
            ] {
                fs::create_dir(trace_dir.join(sub_dir)).unwrap_or_else(|err| {
                    panic!("Failed to create <trace>/{} directory: {}", sub_dir, err)
                });
            }
            self.trace_dir = Some(trace_dir);
        }
    }

    /// Creates an executor with only the standard library Move modules published and not other
    /// initialization done.
    pub fn stdlib_only_genesis() -> Self {
        // let mut genesis = Self::no_genesis();
        // let blobs = cached_framework_packages::module_blobs();
        // let modules = cached_framework_packages::modules();
        // assert!(blobs.len() == modules.len());
        // for (module, bytes) in modules.iter().zip(blobs) {
        //     let id = module.self_id();
        //     genesis.add_module(&id, bytes.to_vec());
        // }
        // genesis
        // TODO(BobOng): e2e-test
        Self::no_genesis()
    }

    /// Creates fresh genesis from the stdlib modules passed in.
    pub fn custom_genesis(
        _genesis_modules: &[Vec<u8>],
        _validator_accounts: Option<usize>,
        _publishing_options: VMConfig,
    ) -> Self {
        // let genesis = vm_genesis::generate_test_genesis(
        //     genesis_modules,
        //     publishing_options,
        //     validator_accounts,
        // );
        // Self::from_genesis(genesis.0.write_set())
        // TODO(BobOng): e2e-test
        Self::no_genesis()
    }

    /// Create one instance of [`AccountData`] without saving it to data store.
    pub fn create_raw_account(&mut self) -> Account {
        Account::new_from_seed(&mut self.rng)
    }

    /// Create one instance of [`AccountData`] without saving it to data store.
    pub fn create_raw_account_data(&mut self, balance: u64, seq_num: u64) -> AccountData {
        AccountData::new_from_seed(&mut self.rng, balance, seq_num)
    }

    /// Creates a number of [`Account`] instances all with the same balance and sequence number,
    /// and publishes them to this executor's data store.
    pub fn create_accounts(&mut self, size: usize, balance: u64, seq_num: u64) -> Vec<Account> {
        let mut accounts: Vec<Account> = Vec::with_capacity(size);
        for _i in 0..size {
            let account_data = AccountData::new_from_seed(&mut self.rng, balance, seq_num);
            self.add_account_data(&account_data);
            accounts.push(account_data.into_account());
        }
        accounts
    }

    /// Applies a [`WriteSet`] to this executor's data store.
    pub fn apply_write_set(&mut self, write_set: &WriteSet) {
        self.data_store
            .apply_write_set(write_set.clone())
            .expect("Panic for cannot apply write set by calling FakeExecutor::apply_write_set");
    }

    /// Adds an account to this executor's data store.
    pub fn add_account_data(&mut self, account_data: &AccountData) {
        //self.data_store.add_account_data(account_data)
        self.apply_write_set(&account_data.to_writeset())
    }

    /// Adds a module to this executor's data store.
    ///
    /// Does not do any sort of verification on the module.
    pub fn add_module(&mut self, module_id: &ModuleId, module_blob: Vec<u8>) {
        self.data_store.add_module(module_id, module_blob)
        // self.data_store
        //     .set(&AccessPath::from(module_id), module_blob)
        //     .expect("Panic for cannot apply write set by calling FakeExecutor::add_module");
    }

    /// Reads the resource [`Value`] for an account from this executor's data store.
    pub fn read_account_resource(&self, account: &Account) -> Option<AccountResource> {
        self.read_account_resource_at_address(account.address())
    }

    fn read_resource<T: MoveResource + for<'a> Deserialize<'a>>(
        &self,
        addr: &AccountAddress,
    ) -> Option<T> {
        let ap = AccessPath::resource_access_path(*addr, T::struct_tag());
        let data_blob = StateView::get_state_value(&self.data_store, &StateKey::AccessPath(ap))
            .expect("account must exist in data store")
            .unwrap_or_else(|| panic!("Can't fetch {} resource for {}", T::STRUCT_NAME, addr));
        bcs::from_bytes(data_blob.as_slice()).ok()
    }

    /// Reads the resource [`Value`] for an account under the given address from
    /// this executor's data store.
    pub fn read_account_resource_at_address(
        &self,
        addr: &AccountAddress,
    ) -> Option<AccountResource> {
        self.read_resource(addr)
    }

    /// Reads the CoinStore resource value for an account from this executor's data store.
    pub fn read_coin_store_resource(&self, account: &Account) -> Option<BalanceResource> {
        self.read_coin_store_resource_at_address(account.address())
    }

    /// Reads the CoinStore resource value for an account under the given address from this executor's
    /// data store.
    pub fn read_coin_store_resource_at_address(
        &self,
        addr: &AccountAddress,
    ) -> Option<BalanceResource> {
        self.read_resource(addr)
    }

    /// Executes the given block of transactions.
    ///
    /// Typical tests will call this method and check that the output matches what was expected.
    /// However, this doesn't apply the results of successful transactions to the data store.
    pub fn execute_block(
        &self,
        txn_block: Vec<SignedUserTransaction>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        self.execute_transaction_block(
            txn_block
                .into_iter()
                .map(Transaction::UserTransaction)
                .collect(),
        )
    }

    /// Alternate form of 'execute_block' that keeps the vm_status before it goes into the
    /// `TransactionOutput`
    pub fn execute_block_and_keep_vm_status(
        &self,
        txn_block: Vec<SignedUserTransaction>,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        StarcoinVM::execute_block_and_keep_vm_status(
            txn_block
                .into_iter()
                .map(Transaction::UserTransaction)
                .collect(),
            &self.data_store,
            None,
            None,
        )
    }

    /// Executes the transaction as a singleton block and applies the resulting write set to the
    /// data store. Panics if execution fails
    pub fn execute_and_apply(&mut self, transaction: SignedUserTransaction) -> TransactionOutput {
        let mut outputs = self.execute_block(vec![transaction]).unwrap();
        assert!(outputs.len() == 1, "transaction outputs size mismatch");
        let output = outputs.pop().unwrap();
        match output.status() {
            TransactionStatus::Keep(status) => {
                self.apply_write_set(output.write_set());
                assert_eq!(
                    status,
                    &KeptVMStatus::Executed,
                    "transaction failed with {:?}",
                    status
                );
                output
            }
            TransactionStatus::Discard(status) => panic!("transaction discarded with {:?}", status),
            TransactionStatus::Retry => panic!("transaction status is retry"),
        }
    }

    pub fn execute_transaction_block_parallel(
        &self,
        txn_block: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        let (result, _) = ParallelStarcoinVM::execute_block(
            txn_block,
            &self.data_store,
            num_cpus::get(),
            None,
            None,
        )?;
        Ok(result)
    }

    pub fn execute_transaction_block(
        &self,
        txn_block: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        let mut trace_map = TraceSeqMapping::default();

        // dump serialized transaction details before execution, if tracing
        if let Some(trace_dir) = &self.trace_dir {
            let trace_data_dir = trace_dir.join(TRACE_DIR_DATA);
            trace_map.0 = Self::trace(trace_data_dir.as_path(), self.get_state_view());
            let trace_input_dir = trace_dir.join(TRACE_DIR_INPUT);
            for txn in &txn_block {
                let input_seq = Self::trace(trace_input_dir.as_path(), txn);
                trace_map.1.push(input_seq);
            }
        }

        let output =
            StarcoinVM::execute_block(txn_block.clone(), &self.get_state_view(), None, None);
        let parallel_output = self.execute_transaction_block_parallel(txn_block);
        assert_eq!(output, parallel_output);

        if let Some(logger) = &self.executed_output {
            logger.log(format!("{:?}\n", output).as_str());
        }

        // dump serialized transaction output after execution, if tracing
        if let Some(trace_dir) = &self.trace_dir {
            match &output {
                Ok(results) => {
                    let trace_output_dir = trace_dir.join(TRACE_DIR_OUTPUT);
                    for res in results {
                        let output_seq = Self::trace(trace_output_dir.as_path(), res);
                        trace_map.2.push(output_seq);
                    }
                }
                Err(e) => {
                    let mut error_file = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(trace_dir.join(TRACE_FILE_ERROR))
                        .unwrap();
                    error_file.write_all(e.to_string().as_bytes()).unwrap();
                }
            }
            let trace_meta_dir = trace_dir.join(TRACE_DIR_META);
            Self::trace(trace_meta_dir.as_path(), &trace_map);
        }
        output
    }

    pub fn execute_transaction(&self, txn: SignedUserTransaction) -> TransactionOutput {
        let txn_block = vec![txn];
        let mut outputs = self
            .execute_block(txn_block)
            .expect("The VM should not fail to startup");
        outputs
            .pop()
            .expect("A block with one transaction should have one output")
    }

    fn trace<P: AsRef<Path>, T: Serialize>(dir: P, item: &T) -> usize {
        let dir = dir.as_ref();
        let seq = fs::read_dir(dir).expect("Unable to read trace dir").count();
        let bytes = bcs::to_bytes(item)
            .unwrap_or_else(|err| panic!("Failed to serialize the trace item: {:?}", err));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(seq.to_string()))
            .expect("Unable to create a trace file");
        file.write_all(&bytes)
            .expect("Failed to write to the trace file");
        seq
    }

    /// Get the blob for the associated AccessPath
    pub fn read_from_access_path(&self, path: &AccessPath) -> Option<Vec<u8>> {
        StateView::get_state_value(&self.data_store, &StateKey::AccessPath(path.clone())).unwrap()
    }

    /// Verifies the given transaction by running it through the VM verifier.
    pub fn verify_transaction(&self, txn: SignedUserTransaction) -> Option<VMStatus> {
        // TODO(BobOng): e2e-test
        //let vm = StarcoinVM::new(self.get_state_view());
        let mut vm = StarcoinVM::new(None);
        vm.verify_transaction(self.get_state_view(), txn)
    }

    pub fn get_state_view(&self) -> &FakeDataStore {
        &self.data_store
    }

    pub fn new_block(&mut self) {
        self.new_block_with_timestamp(self.block_time + 1);
    }

    pub fn new_block_with_timestamp(&mut self, time_stamp: u64) {
        // let validator_set = ValidatorSet::fetch_config(&self.data_store.as_move_resolver())
        //     .expect("Unable to retrieve the validator set from storage");
        // TODO(BobOng): e2e-test
        self.block_time = time_stamp;
        let minter_account = AccountData::new(10000, 0);
        let new_block = BlockMetadata::new(
            HashValue::zero(),
            self.block_time,
            minter_account.address().clone(),
            Some(AuthenticationKey::ed25519(&minter_account.account().pubkey)),
            0,
            0,
            ChainId::test(),
            0,
        );
        let output = self
            .execute_transaction_block(vec![Transaction::BlockMetadata(new_block)])
            .expect("Executing block prologue should succeed")
            .pop()
            .expect("Failed to get the execution result for Block Prologue");
        // check if we emit the expected event, there might be more events for transaction fees
        let event = output.events()[0].clone();
        // TODO(BobOng): e2e-test
        //assert_eq!(event.key(), &new_block_event_key());
        assert!(bcs::from_bytes::<NewBlockEvent>(event.event_data()).is_ok());
        self.apply_write_set(output.write_set());
    }

    fn module(name: &str) -> ModuleId {
        ModuleId::new(CORE_CODE_ADDRESS, Identifier::new(name).unwrap())
    }

    fn name(name: &str) -> Identifier {
        Identifier::new(name).unwrap()
    }

    pub fn set_block_time(&mut self, new_block_time: u64) {
        self.block_time = new_block_time;
    }

    pub fn get_block_time(&mut self) -> u64 {
        self.block_time
    }

    pub fn exec(
        &mut self,
        module_name: &str,
        function_name: &str,
        type_params: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Result<(), VMStatus> {
        let write_set = {
            let gas_params = StarcoinGasParameters::initial();
            let vm = MoveVmExt::new(gas_params.natives.clone()).unwrap();
            let remote_view = RemoteStorage::new(&self.data_store);

            let balance = gas_params.txn.maximum_number_of_gas_units.clone();
            let mut gas_meter = StarcoinGasMeter::new(gas_params, balance);
            gas_meter.set_metering(false);

            let mut session = vm.new_session(&remote_view, SessionId::void());
            session
                .execute_function_bypass_visibility(
                    &Self::module(module_name),
                    &Self::name(function_name),
                    type_params,
                    args,
                    &mut gas_meter,
                )
                .unwrap_or_else(|e| {
                    panic!(
                        "Error calling {}.{}: {}",
                        module_name,
                        function_name,
                        e.into_vm_status()
                    )
                });

            let (change_set, events, mut extensions) = session
                .finish_with_extensions()
                .expect("Failed to generate txn effects");
            let table_context: NativeTableContext = extensions.remove();
            let table_change_set = table_context
                .into_change_set()
                .map_err(|e| e.finish(Location::Undefined))?;

            let (_table_infos, write_set, _events) = SessionOutput {
                change_set,
                events,
                table_change_set,
            }
            .into_change_set(&mut ())?;
            // let (write_set, _events) = session_out
            //     .into_change_set(&mut ())
            //     .expect("Failed to generate writeset")
            //     .into_inner();
            // write_set
            write_set
        };
        self.data_store.add_write_set(&write_set);
        Ok(())
    }

    pub fn try_exec(
        &mut self,
        module_name: &str,
        function_name: &str,
        type_params: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Result<WriteSet, VMStatus> {
        let gas_params = StarcoinGasParameters::initial();
        let vm = MoveVmExt::new(gas_params.natives.clone()).unwrap();
        let remote_view = RemoteStorage::new(&self.data_store);

        let balance = gas_params.txn.maximum_number_of_gas_units.clone();
        let mut gas_meter = StarcoinGasMeter::new(gas_params, balance);
        gas_meter.set_metering(false);

        let mut session = vm.new_session(&remote_view, SessionId::void());
        session
            .execute_function_bypass_visibility(
                &Self::module(module_name),
                &Self::name(function_name),
                type_params,
                args,
                &mut gas_meter,
            )
            .map_err(|e| e.into_vm_status())?;

        let (change_set, events, mut extensions) = session
            .finish_with_extensions()
            .expect("Failed to generate txn effects");

        let table_context: NativeTableContext = extensions.remove();
        let table_change_set = table_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;
        let (_table_infos, write_set, _events) = SessionOutput {
            change_set,
            events,
            table_change_set,
        }
        .into_change_set(&mut ())?;

        // let (writeset, _events) = session_out
        //     .into_change_set(&mut ())
        //     .expect("Failed to generate writeset")
        //     .into_inner();
        Ok(write_set)
    }
}
