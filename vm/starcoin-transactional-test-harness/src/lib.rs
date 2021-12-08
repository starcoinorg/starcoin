use anyhow::{bail, Result};
use itertools::Itertools;
use move_binary_format::{file_format::CompiledScript, CompiledModule};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::GasAlgebra,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, TypeTag},
    transaction_argument::TransactionArgument,
};
use move_lang::compiled_unit::CompiledUnitEnum;
use move_lang::{shared::verify_and_create_named_address_mapping, FullyCompiledProgram};
use move_transactional_test_runner::{
    framework,
    framework::{CompiledState, MoveTestAdapter},
    tasks::{InitCommand, RawAddress, SyntaxChoice, TaskInput},
    vm_test_harness::view_resource_in_move_storage,
};
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    HashValue, ValidCryptoMaterialStringExt,
};
use starcoin_dev::playground::call_contract;
use starcoin_genesis::Genesis;
use starcoin_rpc_api::types::{ContractCall, FunctionIdView, TransactionArgumentView, TypeTagView};
use starcoin_state_api::{ChainStateWriter, StateReaderExt};
use starcoin_statedb::ChainStateDB;
use starcoin_types::{
    access_path::AccessPath,
    account_config::{genesis_address, AccountResource},
    transaction::RawUserTransaction,
};
use starcoin_vm_runtime::{data_cache::RemoteStorage, starcoin_vm::StarcoinVM};
use starcoin_vm_types::write_set::{WriteOp, WriteSetMut};
use starcoin_vm_types::{
    account_config::BalanceResource,
    block_metadata::BlockMetadata,
    genesis_config::ChainId,
    move_resource::MoveResource,
    on_chain_config::VMConfig,
    on_chain_resource,
    state_view::StateView,
    token::{stc::stc_type_tag, token_code::TokenCode},
    transaction::{Module, Script, ScriptFunction, Transaction, TransactionStatus},
    transaction_argument::convert_txn_args,
    vm_status::KeptVMStatus,
};
use std::{collections::BTreeMap, convert::TryInto, path::Path, str::FromStr};
use stdlib::{starcoin_framework_named_addresses, PRECOMPILED_STARCOIN_FRAMEWORK};
use structopt::StructOpt;

fn parse_ed25519_private_key(s: &str) -> Result<Ed25519PrivateKey> {
    Ok(Ed25519PrivateKey::from_encoded_string(s)?)
}

fn parse_named_private_key(s: &str) -> Result<(Identifier, Ed25519PrivateKey)> {
    let before_after = s.split('=').collect::<Vec<_>>();

    if before_after.len() != 2 {
        bail!("Invalid named private key assignment. Must be of the form <private_key_name>=<private_key>, but found '{}'", s);
    }

    let name = Identifier::new(before_after[0])
        .map_err(|_| anyhow::format_err!("Invalid private key name '{}'", s))?;
    let private_key = parse_ed25519_private_key(before_after[1])?;

    Ok((name, private_key))
}

/// A raw private key -- either a literal or an unresolved name.
#[derive(Debug)]
enum RawPrivateKey {
    Named(Identifier),
    Anonymous(Ed25519PrivateKey),
}
impl RawPrivateKey {
    fn parse(s: &str) -> Result<Self> {
        if let Ok(private_key) = parse_ed25519_private_key(s) {
            return Ok(Self::Anonymous(private_key));
        }
        let name = Identifier::new(s)
            .map_err(|_| anyhow::format_err!("Failed to parse '{}' as private key.", s))?;
        Ok(Self::Named(name))
    }
}
impl FromStr for RawPrivateKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(StructOpt, Debug)]
pub struct ExtraInitArgs {
    #[structopt(name = "net", long = "net", short = "n")]
    network: Option<BuiltinNetworkID>,
    #[structopt(long = "private-keys", parse(try_from_str = parse_named_private_key))]
    private_keys: Option<Vec<(Identifier, Ed25519PrivateKey)>>,
}

/// Starcoin-specific arguments for the publish command.
#[derive(StructOpt, Debug)]
pub struct StarcoinPublishArgs {
    #[structopt(short = "k", long = "private-key")]
    privkey: Option<RawPrivateKey>,
}

/// Starcoin-specifc arguments for the run command,
#[derive(StructOpt, Debug)]
pub struct StarcoinRunArgs {
    #[structopt(short = "k", long = "private-key")]
    privkey: Option<RawPrivateKey>,
}

#[derive(StructOpt, Debug)]
pub enum StarcoinSubcommands {
    #[structopt(name = "block")]
    NewBlock {
        #[structopt(long)]
        author: Option<AccountAddress>,
        timestamp: Option<u64>,
        number: Option<u64>,
        uncles: Option<u64>,
    },
    #[structopt(name = "call")]
    ContractCall {
        #[structopt(name = "FUNCTION")]
        name: FunctionIdView,
        #[structopt(long = "args", short = "i")]
        args: Vec<TransactionArgumentView>,
        #[structopt(long = "type-args", short = "t")]
        type_args: Vec<TypeTagView>,
    },
}

pub struct StarcoinTestAdapter<'a> {
    compiled_state: CompiledState<'a>,
    storage: ChainStateDB,
    default_syntax: SyntaxChoice,
    private_key_mapping: BTreeMap<Identifier, Ed25519PrivateKey>,
}

/// Parameters *required* to create a Starcoin transaction.
struct TransactionParameters {
    pub sequence_number: u64,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub expiration_timestamp_secs: u64,
}

impl<'a> StarcoinTestAdapter<'a> {
    /// Look up the named private key in the mapping.
    fn resolve_named_private_key(&self, s: &IdentStr) -> Ed25519PrivateKey {
        if let Some(private_key) = self.private_key_mapping.get(s) {
            return private_key.clone();
        }
        panic!("Failed to resolve private key '{}'", s)
    }

    /// Resolve a raw private key into a numeric one.
    fn resolve_private_key(&self, private_key: &RawPrivateKey) -> Ed25519PrivateKey {
        match private_key {
            RawPrivateKey::Anonymous(private_key) => private_key.clone(),
            RawPrivateKey::Named(name) => self.resolve_named_private_key(name),
        }
    }

    /// Obtain a Rust representation of the account resource from storage, which is used to derive
    /// a few default transaction parameters.
    fn fetch_account_resource(&self, signer_addr: &AccountAddress) -> Result<AccountResource> {
        let account_access_path =
            AccessPath::resource_access_path(*signer_addr, AccountResource::struct_tag());
        let account_blob = self.storage.get(&account_access_path)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to fetch account resource under address {}. Has the account been created?",
                signer_addr
            )
        })?;
        Ok(bcs::from_bytes(&account_blob).unwrap())
    }

    /// Obtain a Rust representation of the balance resource from storage, which is used to derive
    /// a few default transaction parameters.
    fn fetch_balance_resource(
        &self,
        signer_addr: &AccountAddress,
        balance_currency_code: Identifier,
    ) -> Result<BalanceResource> {
        let token_code = TokenCode::from_str(balance_currency_code.as_str())?;
        let balance_resource_tag = BalanceResource::struct_tag_for_token(token_code.try_into()?);
        let balance_access_path =
            AccessPath::resource_access_path(*signer_addr, balance_resource_tag);

        let balance_blob = self.storage.get(&balance_access_path)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to fetch balance resource under address {}.",
                signer_addr
            )
        })?;

        Ok(bcs::from_bytes(&balance_blob).unwrap())
    }

    /// Derive the default transaction parameters from the account and balance resources fetched
    /// from storage. In the future, we are planning to allow the user to override these using
    /// command arguments.
    fn fetch_default_transaction_parameters(
        &self,
        signer_addr: &AccountAddress,
    ) -> Result<TransactionParameters> {
        let account_resource = self.fetch_account_resource(signer_addr)?;

        let sequence_number = account_resource.sequence_number();
        let gas_currency_code = Identifier::new(stc_type_tag().to_string())?;
        let vmconfig = self
            .storage
            .get_on_chain_config::<VMConfig>()?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch onchain vm config."))?;
        let max_number_of_gas_units = vmconfig
            .gas_schedule
            .gas_constants
            .maximum_number_of_gas_units;
        let gas_unit_price = 0;
        let max_gas_amount = if gas_unit_price == 0 {
            max_number_of_gas_units.get()
        } else {
            let account_balance =
                self.fetch_balance_resource(signer_addr, gas_currency_code.clone())?;
            std::cmp::min(
                max_number_of_gas_units.get(),
                (account_balance.token() / gas_unit_price as u128) as u64,
            )
        };

        Ok(TransactionParameters {
            sequence_number,
            gas_unit_price,
            max_gas_amount,
            expiration_timestamp_secs: self.storage.get_timestamp()?.seconds() + 60 * 60,
        })
    }

    /// Perform a single Starcoin transaction.
    ///
    /// Should error if the transaction ends up being discarded, or having a status other than
    /// EXECUTED.
    fn run_transaction(&mut self, txns: Vec<Transaction>) -> Result<()> {
        let mut vm = StarcoinVM::new(None);
        let mut outputs = vm.execute_block_transactions(&self.storage, txns, None)?;

        assert_eq!(outputs.len(), 1);

        let (status, output) = outputs.pop().unwrap();
        match output.status() {
            TransactionStatus::Keep(kept_vm_status) => match kept_vm_status {
                KeptVMStatus::Executed => {
                    self.storage.apply_write_set(output.into_inner().0)?;
                }
                _ => {
                    bail!("Failed to execute transaction. VMStatus: {}", status)
                }
            },
            TransactionStatus::Discard(_) => {
                bail!("Transaction discarded. VMStatus: {}", status)
            }
        }

        Ok(())
    }

    fn handle_contract_call(&self, call: ContractCall) -> Result<Option<String>> {
        let ContractCall {
            function_id,
            type_args,
            args,
        } = call;
        let rets = call_contract(
            &self.storage,
            function_id.0.module,
            function_id.0.function.as_str(),
            type_args.into_iter().map(|t| t.0).collect(),
            args.into_iter().map(|t| t.0).collect(),
            None,
        )?;

        let move_resolver = RemoteStorage::new(&self.storage);
        let annotator = resource_viewer::MoveValueAnnotator::new(&move_resolver);
        let rets = rets
            .into_iter()
            .map(|(ty, v)| annotator.view_value(&ty, &v))
            .collect::<Result<Vec<_>>>()?;
        Ok(Some(rets.iter().map(|t| format!("{}", t)).join("\n")))
    }

    fn handle_new_block(
        &mut self,
        author: Option<AccountAddress>,
        timestamp: Option<u64>,
        number: Option<u64>,
        uncles: Option<u64>,
    ) -> Result<Option<String>> {
        let last_blockmeta = self
            .storage
            .get_resource::<on_chain_resource::BlockMetadata>(genesis_address())?;

        let height = number
            .or(last_blockmeta.as_ref().map(|b| b.number + 1))
            .unwrap_or(0);

        let author = author
            .or(last_blockmeta.as_ref().map(|b| b.author))
            .unwrap_or_else(AccountAddress::random);

        let uncles = uncles
            .or(last_blockmeta.as_ref().map(|b| b.uncles))
            .unwrap_or(0);
        let timestamp = timestamp.unwrap_or(self.storage.get_timestamp()?.milliseconds + 10 * 1000);
        let new_block_meta = BlockMetadata::new(
            HashValue::random(),
            timestamp,
            author,
            None,
            uncles,
            height,
            self.storage.get_chain_id()?,
            0,
        );
        self.run_transaction(vec![Transaction::BlockMetadata(new_block_meta)])?;
        Ok(None)
    }
}
fn panic_missing_private_key_named(cmd_name: &str, name: &IdentStr) -> ! {
    panic!(
        "Missing private key. Either add a `--private-key <priv_key>` argument \
            to the {} command, or associate an address to the \
            name '{}' in the init command.",
        cmd_name, name,
    )
}

fn panic_missing_private_key(cmd_name: &str) -> ! {
    panic!(
        "Missing private key. Try adding a `--private-key <priv_key>` \
            argument to the {} command.",
        cmd_name
    )
}

impl<'a> MoveTestAdapter<'a> for StarcoinTestAdapter<'a> {
    type ExtraPublishArgs = StarcoinPublishArgs;
    type ExtraRunArgs = StarcoinRunArgs;
    type Subcommand = StarcoinSubcommands;
    type ExtraInitArgs = ExtraInitArgs;

    fn compiled_state(&mut self) -> &mut CompiledState<'a> {
        &mut self.compiled_state
    }

    fn default_syntax(&self) -> SyntaxChoice {
        self.default_syntax
    }

    fn init(
        default_syntax: SyntaxChoice,
        pre_compiled_deps: Option<&'a FullyCompiledProgram>,
        task_opt: Option<TaskInput<(InitCommand, Self::ExtraInitArgs)>>,
    ) -> Self {
        let (additional_mapping, _network) = match task_opt.as_ref().map(|t| &t.command) {
            Some((InitCommand { named_addresses }, ExtraInitArgs { network, .. })) => (
                verify_and_create_named_address_mapping(named_addresses.clone()).unwrap(),
                network.clone(),
            ),
            None => (BTreeMap::new(), None),
        };

        let mut named_address_mapping = starcoin_framework_named_addresses();
        for (name, addr) in additional_mapping {
            if named_address_mapping.contains_key(&name) {
                panic!(
                    "Invalid init. The named address '{}' is reserved by either the move-stdlib or Starcoin-framework",
                    name
                )
            }
            named_address_mapping.insert(name, addr);
        }

        let mut private_key_mapping = BTreeMap::default();
        if let Some(TaskInput {
            command: (_, init_args),
            ..
        }) = task_opt
        {
            // Private key mapping
            if let Some(additional_private_key_mapping) = init_args.private_keys {
                for (name, private_key) in additional_private_key_mapping {
                    if private_key_mapping.contains_key(&name) {
                        panic!(
                            "Invalid init. The named private key '{}' already exists.",
                            name
                        )
                    }
                    private_key_mapping.insert(name, private_key);
                }
            }
        }

        let store = {
            let net = ChainNetwork::new_test();
            let genesis_txn = Genesis::build_genesis_transaction(&net).unwrap();
            let data_store = ChainStateDB::mock();
            Genesis::execute_genesis_txn(&data_store, genesis_txn).unwrap();
            data_store
        };

        // add pre compiled modules
        if let Some(pre_compiled_lib) = pre_compiled_deps {
            let mut writes = WriteSetMut::default();
            for c in &pre_compiled_lib.compiled {
                if let CompiledUnitEnum::Module(m) = c {
                    writes.push((
                        AccessPath::code_access_path(
                            m.named_module.address.into_inner(),
                            Identifier::new(m.named_module.name.as_str()).unwrap(),
                        ),
                        WriteOp::Value({
                            let mut bytes = vec![];
                            m.named_module.module.serialize(&mut bytes).unwrap();
                            bytes
                        }),
                    ));
                }
            }
            store.apply_write_set(writes.freeze().unwrap()).unwrap();
        }

        Self {
            compiled_state: CompiledState::new(named_address_mapping, pre_compiled_deps),
            default_syntax,
            private_key_mapping,
            storage: store,
        }
    }

    fn publish_module(
        &mut self,
        module: CompiledModule,
        named_addr_opt: Option<Identifier>,
        gas_budget: Option<u64>,
        extra: Self::ExtraPublishArgs,
    ) -> anyhow::Result<()> {
        let module_id = module.self_id();
        let signer = module_id.address();
        let params = self.fetch_default_transaction_parameters(signer)?;

        let mut module_blob = vec![];
        module.serialize(&mut module_blob).unwrap();

        let private_key = match (extra.privkey, named_addr_opt) {
            (Some(private_key), _) => self.resolve_private_key(&private_key),
            (None, Some(named_addr)) => match self.private_key_mapping.get(&named_addr) {
                Some(private_key) => private_key.clone(),
                None => panic_missing_private_key_named("publish", &named_addr),
            },
            (None, None) => panic_missing_private_key("publish"),
        };

        let txn = RawUserTransaction::new_module(
            *signer,
            params.sequence_number,
            Module::new(module_blob),
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            ChainId::test(),
        )
        .sign(&private_key, Ed25519PublicKey::from(&private_key))?
        .into_inner();

        self.run_transaction(vec![Transaction::UserTransaction(txn)])?;

        Ok(())
    }

    fn execute_script(
        &mut self,
        script: CompiledScript,
        type_args: Vec<TypeTag>,
        signers: Vec<RawAddress>,
        args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
        extra_args: Self::ExtraRunArgs,
    ) -> anyhow::Result<()> {
        assert!(!signers.is_empty());
        if signers.len() != 1 {
            panic!("Expected 1 signer, got {}.", signers.len());
        }
        let sender = self.compiled_state().resolve_address(&signers[0]);

        let mut script_blob = vec![];
        script.serialize(&mut script_blob)?;

        let params = self.fetch_default_transaction_parameters(&sender)?;

        let private_key = match (extra_args.privkey, &signers[0]) {
            (Some(private_key), _) => self.resolve_private_key(&private_key),
            (None, RawAddress::Named(named_addr)) => match self.private_key_mapping.get(named_addr)
            {
                Some(private_key) => private_key.clone(),
                None => panic_missing_private_key_named("run", named_addr),
            },
            (None, RawAddress::Anonymous(_)) => panic_missing_private_key("run"),
        };

        let txn = RawUserTransaction::new_script(
            sender,
            params.sequence_number,
            Script::new(script_blob, type_args, convert_txn_args(&args)),
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            ChainId::test(),
        )
        .sign(&private_key, Ed25519PublicKey::from(&private_key))
        .unwrap()
        .into_inner();

        self.run_transaction(vec![Transaction::UserTransaction(txn)])?;

        Ok(())
    }

    fn call_function(
        &mut self,
        module: &ModuleId,
        function: &IdentStr,
        type_args: Vec<TypeTag>,
        signers: Vec<RawAddress>,
        args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
        extra_args: Self::ExtraRunArgs,
    ) -> anyhow::Result<()> {
        {
            assert!(!signers.is_empty());
            if signers.len() != 1 {
                panic!("Expected 1 signer, got {}.", signers.len());
            }
        }
        let sender = self.compiled_state().resolve_address(&signers[0]);

        let params = self.fetch_default_transaction_parameters(&sender)?;

        let private_key = match (extra_args.privkey, &signers[0]) {
            (Some(private_key), _) => self.resolve_private_key(&private_key),
            (None, RawAddress::Named(named_addr)) => match self.private_key_mapping.get(named_addr)
            {
                Some(private_key) => private_key.clone(),
                None => panic_missing_private_key_named("run", named_addr),
            },
            (None, RawAddress::Anonymous(_)) => panic_missing_private_key("run"),
        };

        let txn = RawUserTransaction::new_script_function(
            sender,
            params.sequence_number,
            ScriptFunction::new(
                module.clone(),
                function.to_owned(),
                type_args,
                convert_txn_args(&args),
            ),
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            ChainId::test(),
        )
        .sign(&private_key, Ed25519PublicKey::from(&private_key))
        .unwrap()
        .into_inner();

        self.run_transaction(vec![Transaction::UserTransaction(txn)])?;

        Ok(())
    }

    fn view_data(
        &mut self,
        address: AccountAddress,
        module: &ModuleId,
        resource: &IdentStr,
        type_args: Vec<TypeTag>,
    ) -> anyhow::Result<String> {
        let s = RemoteStorage::new(&self.storage);
        view_resource_in_move_storage(&s, address, module, resource, type_args)
    }

    fn handle_subcommand(
        &mut self,
        subcommand: TaskInput<Self::Subcommand>,
    ) -> anyhow::Result<Option<String>> {
        match subcommand.command {
            StarcoinSubcommands::NewBlock {
                author,
                timestamp,
                number,
                uncles,
            } => self.handle_new_block(author, timestamp, number, uncles),
            StarcoinSubcommands::ContractCall {
                name,
                args,
                type_args,
            } => self.handle_contract_call(ContractCall {
                function_id: name,
                args,
                type_args,
            }),
        }
    }
}

/// Run the Starcoin transactional test flow, using the given file as input.
pub fn run_test(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_test_impl(path, Some(&*PRECOMPILED_STARCOIN_FRAMEWORK))
}

pub fn run_test_impl<'a>(
    path: &Path,
    fully_compiled_program_opt: Option<&'a FullyCompiledProgram>,
) -> Result<(), Box<dyn std::error::Error>> {
    framework::run_test_impl::<StarcoinTestAdapter>(path, fully_compiled_program_opt)
}
