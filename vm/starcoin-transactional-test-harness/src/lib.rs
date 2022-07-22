use crate::in_memory_state_cache::InMemoryStateCache;
use crate::remote_state::{RemoteViewer, SelectableStateView};
use anyhow::{bail, Result};
use move_binary_format::{file_format::CompiledScript, CompiledModule};
use move_compiler::compiled_unit::CompiledUnitEnum;
use move_compiler::shared::{NumberFormat, NumericalAddress};
use move_compiler::{shared::verify_and_create_named_address_mapping, FullyCompiledProgram};
use move_core_types::language_storage::StructTag;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::GasAlgebra,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, TypeTag},
    transaction_argument::TransactionArgument,
};
use move_transactional_test_runner::framework;
use move_transactional_test_runner::{
    framework::{CompiledState, MoveTestAdapter},
    tasks::{InitCommand, RawAddress, SyntaxChoice, TaskInput},
    vm_test_harness::view_resource_in_move_storage,
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use starcoin_abi_decoder::decode_txn_payload;
use starcoin_config::{genesis_key_pair, BuiltinNetworkID, ChainNetwork};
use starcoin_crypto::ed25519::Ed25519PrivateKey;
use starcoin_crypto::{HashValue, PrivateKey, ValidCryptoMaterial, ValidCryptoMaterialStringExt};
use starcoin_dev::playground::call_contract;
use starcoin_genesis::Genesis;
use starcoin_rpc_api::types::{
    ContractCall, FunctionIdView, RawUserTransactionView, TransactionArgumentView,
    TransactionOutputView, TransactionStatusView, TypeTagView,
};
use starcoin_rpc_api::Params;
use starcoin_state_api::{ChainStateWriter, StateReaderExt};
use starcoin_statedb::ChainStateDB;
use starcoin_types::account::{Account, AccountData};
use starcoin_types::{
    access_path::AccessPath,
    account_config::{genesis_address, AccountResource},
    transaction::RawUserTransaction,
};
use starcoin_vm_runtime::{data_cache::RemoteStorage, starcoin_vm::StarcoinVM};
use starcoin_vm_types::account_config::{
    association_address, core_code_address, STC_TOKEN_CODE_STR,
};

use clap::Parser;
use starcoin_vm_types::transaction::authenticator::AccountPublicKey;
use starcoin_vm_types::transaction::DryRunTransaction;
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
use std::convert::TryFrom;
use std::{collections::BTreeMap, convert::TryInto, path::Path, str::FromStr};
use stdlib::{starcoin_framework_named_addresses, G_PRECOMPILED_STARCOIN_FRAMEWORK};

mod in_memory_state_cache;
pub mod remote_state;

fn parse_ed25519_key<T: ValidCryptoMaterial>(s: &str) -> Result<T> {
    Ok(T::from_encoded_string(s)?)
}

fn parse_named_key<T: ValidCryptoMaterial>(s: &str) -> Result<(Identifier, T)> {
    let before_after = s.split('=').collect::<Vec<_>>();

    if before_after.len() != 2 {
        bail!(
            "Invalid named key assignment. Must be of the form <key_name>=<key>, but found '{}'",
            s
        );
    }

    let name = Identifier::new(before_after[0])
        .map_err(|_| anyhow::format_err!("Invalid key name '{}'", s))?;
    let key = parse_ed25519_key(before_after[1])?;

    Ok((name, key))
}

/// A raw private key -- either a literal or an unresolved name.
#[derive(Debug)]
enum RawPublicKey {
    Named(Identifier),
    Anonymous(AccountPublicKey),
}
impl RawPublicKey {
    fn parse(s: &str) -> Result<Self> {
        if let Ok(private_key) = parse_ed25519_key(s) {
            return Ok(Self::Anonymous(private_key));
        }
        let name = Identifier::new(s)
            .map_err(|_| anyhow::format_err!("Failed to parse '{}' as private key.", s))?;
        Ok(Self::Named(name))
    }
}
impl FromStr for RawPublicKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Parser, Debug, Default)]
pub struct ExtraInitArgs {
    #[clap(name = "rpc", long)]
    /// use remote starcoin rpc as initial state.
    rpc: Option<String>,
    #[clap(long = "block-number", requires("rpc"))]
    /// block number to read state from. default to latest block number.
    block_number: Option<u64>,

    #[clap(long = "network", short, conflicts_with("rpc"))]
    /// genesis with the network
    network: Option<BuiltinNetworkID>,

    #[clap(long = "public-keys",
    parse(try_from_str = parse_named_key),
    takes_value(true),
    multiple_values(true),
    multiple_occurrences(true))]
    public_keys: Option<Vec<(Identifier, AccountPublicKey)>>,
    // #[clap(long = "private-keys", parse(try_from_str = parse_named_private_key))]
    // private_keys: Option<Vec<(Identifier, Ed25519PrivateKey)>>,
    #[clap(long = "debug")]
    /// enable debug mode, output more info to stderr.
    debug: bool,
}

/// Starcoin-specific arguments for the publish command.
#[derive(Parser, Debug)]
pub struct StarcoinPublishArgs {
    #[clap(short = 'k', long = "public-key")]
    public_key: Option<RawPublicKey>,
}

/// Starcoin-specific arguments for the run command,
#[derive(Parser, Debug)]
pub struct StarcoinRunArgs {
    #[clap(short = 'k', long = "public-key")]
    public_key: Option<RawPublicKey>,
}

#[derive(clap::Args, Debug)]
#[clap(name = "faucet")]
struct FaucetSub {
    #[clap(long="addr", parse(try_from_str=RawAddress::parse))]
    address: RawAddress,
    #[clap(long = "amount", default_value = "100000000000")]
    initial_balance: u128,
    #[clap(long = "public-key", parse(try_from_str=AccountPublicKey::from_encoded_string))]
    public_key: Option<AccountPublicKey>,
}

#[derive(clap::Args, Debug)]
#[clap(name = "block")]
struct BlockSub {
    #[clap(long, parse(try_from_str=RawAddress::parse))]
    author: Option<RawAddress>,
    #[clap(long)]
    timestamp: Option<u64>,
    #[clap(long)]
    number: Option<u64>,
    #[clap(long)]
    uncles: Option<u64>,
}
#[derive(clap::Args, Debug)]
#[clap(name = "call")]
struct CallSub {
    #[clap(name = "FUNCTION")]
    name: FunctionIdView,
    #[clap(long = "args", short = 'i')]
    args: Vec<TransactionArgumentView>,
    #[clap(long = "type-args", short = 't')]
    type_args: Vec<TypeTagView>,
}

#[derive(Debug, Parser)]
#[clap(name = "call-api")]
pub struct CallAPISub {
    #[clap(name = "method")]
    /// api method to call, example: node.info
    method: String,
    #[clap(name = "params", default_value = "", parse(try_from_str=parse_params))]
    /// api params, should be a json array string
    params: Params,
}

#[derive(Parser, Debug)]
pub enum StarcoinSubcommands {
    #[clap(name = "faucet")]
    Faucet {
        #[clap(long="addr", parse(try_from_str=RawAddress::parse))]
        address: RawAddress,
        #[clap(long = "amount", default_value = "100000000000")]
        initial_balance: u128,
        #[clap(long = "public-key", parse(try_from_str=AccountPublicKey::from_encoded_string))]
        public_key: Option<AccountPublicKey>,
    },

    #[clap(name = "block")]
    NewBlock {
        #[clap(long, parse(try_from_str=RawAddress::parse))]
        author: Option<RawAddress>,
        #[clap(long)]
        timestamp: Option<u64>,
        #[clap(long)]
        number: Option<u64>,
        #[clap(long)]
        uncles: Option<u64>,
    },
    #[clap(name = "call")]
    ContractCall {
        #[clap(name = "FUNCTION")]
        name: FunctionIdView,
        #[clap(long = "args", short = 'i')]
        args: Vec<TransactionArgumentView>,
        #[clap(long = "type-args", short = 't')]
        type_args: Vec<TypeTagView>,
    },
    #[clap(name = "call-api")]
    CallAPI {
        #[clap(name = "method")]
        /// api name to call, example: node.info
        method: String,
        #[clap(name = "params", default_value = "", parse(try_from_str=parse_params))]
        /// api params, should be a json array string
        params: Params,
    },
}

fn parse_params(params: &str) -> Result<Params> {
    let params = match params.trim() {
        "" => Params::None,
        param => serde_json::from_str(param)?,
    };
    Ok(params)
}

impl From<FaucetSub> for StarcoinSubcommands {
    fn from(sub: FaucetSub) -> Self {
        Self::Faucet {
            address: sub.address,
            initial_balance: sub.initial_balance,
            public_key: sub.public_key,
        }
    }
}

impl From<BlockSub> for StarcoinSubcommands {
    fn from(sub: BlockSub) -> Self {
        Self::NewBlock {
            author: sub.author,
            timestamp: sub.timestamp,
            number: sub.number,
            uncles: sub.uncles,
        }
    }
}

impl From<CallSub> for StarcoinSubcommands {
    fn from(sub: CallSub) -> Self {
        Self::ContractCall {
            name: sub.name,
            args: sub.args,
            type_args: sub.type_args,
        }
    }
}

impl From<CallAPISub> for StarcoinSubcommands {
    fn from(sub: CallAPISub) -> Self {
        Self::CallAPI {
            method: sub.method,
            params: sub.params,
        }
    }
}

impl clap::Args for StarcoinSubcommands {
    fn augment_args(cmd: clap::Command<'_>) -> clap::Command<'_> {
        let faucet = FaucetSub::augment_args(clap::Command::new("faucet"));
        let block = BlockSub::augment_args(clap::Command::new("block"));
        let call = CallSub::augment_args(clap::Command::new("call"));
        let call_api = CallAPISub::augment_args(clap::Command::new("call-api"));
        cmd.subcommand(faucet)
            .subcommand(block)
            .subcommand(call)
            .subcommand(call_api)
    }

    fn augment_args_for_update(_cmd: clap::Command<'_>) -> clap::Command<'_> {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransactionWithOutput {
    pub txn: RawUserTransactionView,
    pub output: TransactionOutputView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimpleTransactionResult {
    gas_used: u64,
    status: TransactionStatusView,
}

pub struct StarcoinTestAdapter<'a> {
    compiled_state: CompiledState<'a>,
    storage: SelectableStateView<ChainStateDB, InMemoryStateCache<RemoteViewer>>,
    default_syntax: SyntaxChoice,
    public_key_mapping: BTreeMap<Identifier, AccountPublicKey>,
    association_public_key: AccountPublicKey,
    remote_viewer: Option<RemoteViewer>,
    debug: bool,
}

/// Parameters *required* to create a Starcoin transaction.
struct TransactionParameters {
    pub sequence_number: u64,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub expiration_timestamp_secs: u64,
    pub chainid: ChainId,
}

impl<'a> StarcoinTestAdapter<'a> {
    /// Look up the named private key in the mapping.
    fn resolve_named_public_key(&self, s: &IdentStr) -> AccountPublicKey {
        if let Some(public_key) = self.public_key_mapping.get(s) {
            return public_key.clone();
        }
        panic!("Failed to resolve private key '{}'", s)
    }

    /// Resolve a raw public key into a numeric one.
    fn resolve_public_key(&self, public_key: &RawPublicKey) -> AccountPublicKey {
        match public_key {
            RawPublicKey::Anonymous(public_key) => public_key.clone(),
            RawPublicKey::Named(name) => self.resolve_named_public_key(name),
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
        balance_currency_code: String,
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

    fn hack_genesis_account(&self) -> Result<()> {
        let genesis_account = self.fetch_account_resource(&genesis_address())?;

        let balance =
            self.fetch_balance_resource(&genesis_address(), STC_TOKEN_CODE_STR.to_string())?;
        let genesis_account_data = AccountData::with_account_and_event_counts(
            Account::new_genesis_account(genesis_address()),
            balance.token(),
            STC_TOKEN_CODE_STR,
            genesis_account.sequence_number(),
            genesis_account.withdraw_events().count(),
            genesis_account.deposit_events().count(),
            genesis_account.accept_token_events().count(),
            genesis_account.has_delegated_key_rotation_capability(),
            genesis_account.has_delegated_withdrawal_capability(),
        );
        self.storage
            .apply_write_set(genesis_account_data.to_writeset())?;

        {
            let mut writes = WriteSetMut::default();
            writes.push((
                AccessPath::resource_access_path(
                    genesis_address(),
                    StructTag {
                        address: genesis_address(),
                        module: Identifier::new("Account")?,
                        name: Identifier::new("SignerDelegated")?,
                        type_params: vec![],
                    },
                ),
                WriteOp::Deletion,
            ));
            self.storage.apply_write_set(writes.freeze().unwrap())?;
        }
        Ok(())
    }

    /// Hack the account, and set account's auth key to genesis keypair
    fn hack_account(&self, address: AccountAddress) -> Result<()> {
        let account = self.fetch_account_resource(&address)?;

        let balance = self.fetch_balance_resource(&address, STC_TOKEN_CODE_STR.to_string())?;
        let account_data = AccountData::with_account_and_event_counts(
            Account::new_genesis_account(address),
            balance.token(),
            STC_TOKEN_CODE_STR,
            account.sequence_number(),
            account.withdraw_events().count(),
            account.deposit_events().count(),
            account.accept_token_events().count(),
            account.has_delegated_key_rotation_capability(),
            account.has_delegated_withdrawal_capability(),
        );
        self.storage.apply_write_set(account_data.to_writeset())?;
        Ok(())
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
        // let gas_currency_code = stc_type_tag().to_string();
        let vmconfig = self
            .storage
            .get_on_chain_config::<VMConfig>()?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch onchain vm config."))?;
        let max_number_of_gas_units = vmconfig
            .gas_schedule
            .gas_constants
            .maximum_number_of_gas_units;
        let gas_unit_price = 1;
        let max_gas_amount = if gas_unit_price == 0 {
            max_number_of_gas_units.get()
        } else {
            let account_balance =
                self.fetch_balance_resource(signer_addr, stc_type_tag().to_string())?;
            std::cmp::min(
                max_number_of_gas_units.get(),
                (account_balance.token() / gas_unit_price as u128) as u64,
            )
        };
        let chain_id = self.storage.get_chain_id()?;
        Ok(TransactionParameters {
            sequence_number,
            gas_unit_price,
            max_gas_amount,
            expiration_timestamp_secs: self.storage.get_timestamp()?.seconds() + 60 * 60,
            chainid: chain_id,
        })
    }

    /// Perform a single Starcoin transaction.
    ///
    /// Should error if the transaction ends up being discarded, or having a status other than
    /// EXECUTED.
    fn run_blockmeta(&mut self, meta: BlockMetadata) -> Result<()> {
        let mut vm = StarcoinVM::new(None);
        let mut outputs = vm.execute_block_transactions(
            &self.storage,
            vec![Transaction::BlockMetadata(meta)],
            None,
        )?;
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

    /// Perform a single Starcoin transaction.
    ///
    /// Should error if the transaction ends up being discarded, or having a status other than
    /// EXECUTED.
    fn run_transaction(
        &mut self,
        txn: RawUserTransaction,
        public_key: AccountPublicKey,
    ) -> Result<TransactionWithOutput> {
        let mut vm = StarcoinVM::new(None);
        let (_status, output) = vm.dry_run_transaction(
            &self.storage,
            DryRunTransaction {
                raw_txn: txn.clone(),
                public_key,
            },
        )?;
        match output.status() {
            TransactionStatus::Keep(_kept_vm_status) => {
                self.storage
                    .apply_write_set(output.clone().into_inner().0)?;
            }
            TransactionStatus::Discard(_) => {}
        }
        let payload = decode_txn_payload(&self.storage, txn.payload())?;
        let mut txn_view: RawUserTransactionView = txn.try_into()?;
        txn_view.decoded_payload = Some(payload.into());
        Ok(TransactionWithOutput {
            txn: txn_view,
            output: output.into(),
        })
    }

    fn handle_contract_call(&self, call: ContractCall) -> Result<(Option<String>, Option<Value>)> {
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
        let annotator = move_resource_viewer::MoveValueAnnotator::new(&move_resolver);
        let rets = rets
            .into_iter()
            .map(|(ty, v)| annotator.view_value(&ty, &v))
            .collect::<Result<Vec<_>>>()?;
        if rets.is_empty() {
            Ok((None, None))
        } else if rets.len() == 1 {
            Ok((
                Some(serde_json::to_string_pretty(&rets[0])?),
                Some(serde_json::to_value(&rets[0])?),
            ))
        } else {
            Ok((
                Some(serde_json::to_string_pretty(&rets)?),
                Some(serde_json::to_value(&rets)?),
            ))
        }
    }

    fn handle_faucet(
        &mut self,
        addr: RawAddress,
        initial_balance: u128,
        public_key: Option<AccountPublicKey>,
    ) -> Result<(Option<String>, Option<Value>)> {
        let sender = association_address();

        let params = self.fetch_default_transaction_parameters(&sender)?;

        match &addr {
            RawAddress::Named(name) => {
                if !self.compiled_state.contain_name_address(name.as_str()) {
                    // make it deterministic.
                    let key = Ed25519PrivateKey::try_from(
                        HashValue::sha3_256_of(name.as_bytes()).as_slice(),
                    )
                    .unwrap()
                    .public_key();

                    let addr = AccountPublicKey::Single(key.clone()).derived_address();
                    self.compiled_state
                        .add_named_addresses({
                            let mut addrs = BTreeMap::default();
                            addrs.insert(
                                name.as_str(),
                                NumericalAddress::new(addr.into_bytes(), NumberFormat::Hex),
                            );
                            addrs
                        })
                        .unwrap();
                    self.public_key_mapping.insert(name.clone(), key.into());
                } else if public_key.is_some() {
                    bail!(
                        "name address {} = {} already exists, should not provide public key",
                        name,
                        self.compiled_state.resolve_address(&addr)
                    );
                }
            }
            RawAddress::Anonymous(addr) => {
                if let Some(_public_key) = public_key {
                    bail!(
                        "create anonymous address {} cannot provide public key",
                        addr
                    );
                }
            }
        }

        let addr = self.compiled_state.resolve_address(&addr);
        let txn = RawUserTransaction::new_script_function(
            sender,
            params.sequence_number,
            ScriptFunction::new(
                ModuleId::new(
                    core_code_address(),
                    Identifier::new("TransferScripts").unwrap(),
                ),
                Identifier::new("peer_to_peer_v2").unwrap(),
                vec![stc_type_tag()],
                vec![
                    bcs_ext::to_bytes(&addr).unwrap(),
                    bcs_ext::to_bytes(&initial_balance).unwrap(),
                ],
            ),
            params.max_gas_amount,
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            params.chainid,
        );
        let output = self.run_transaction(txn, self.association_public_key.clone())?;

        match output.output.status {
            TransactionStatusView::Executed => Ok((None, None)),
            _ => {
                bail!(
                    "Failed to faucet {}, status: {:?}",
                    addr,
                    output.output.status
                );
            }
        }
    }
    fn handle_new_block(
        &mut self,
        author: Option<RawAddress>,
        timestamp: Option<u64>,
        number: Option<u64>,
        uncles: Option<u64>,
    ) -> Result<(Option<String>, Option<Value>)> {
        let last_blockmeta = self
            .storage
            .get_resource::<on_chain_resource::BlockMetadata>(genesis_address())?;

        let height = number
            .or_else(|| last_blockmeta.as_ref().map(|b| b.number + 1))
            .unwrap_or(0);

        let author = author
            .map(|v| self.compiled_state.resolve_address(&v))
            .or_else(|| last_blockmeta.as_ref().map(|b| b.author))
            .unwrap_or_else(AccountAddress::random);

        let uncles = uncles
            .or_else(|| last_blockmeta.as_ref().map(|b| b.uncles))
            .unwrap_or(0);
        let timestamp = timestamp.unwrap_or(self.storage.get_timestamp()?.milliseconds + 10 * 1000);
        //TODO find a better way to get parent hash, we should keep to local storage.
        let parent_hash = last_blockmeta
            .as_ref()
            .map(|b| {
                let mut parent_hash = b.parent_hash.to_vec();
                parent_hash.extend(bcs_ext::to_bytes(&b.number).expect("bcs should success"));
                HashValue::sha3_256_of(parent_hash.as_slice())
            })
            .unwrap_or_else(HashValue::zero);
        let new_block_meta = BlockMetadata::new(
            parent_hash,
            timestamp,
            author,
            None,
            uncles,
            height,
            self.storage.get_chain_id()?,
            0,
        );
        self.run_blockmeta(new_block_meta.clone())?;

        Ok((None, Some(serde_json::to_value(&new_block_meta)?)))
    }

    fn handle_call_api(
        &mut self,
        method: String,
        params: Params,
    ) -> Result<(Option<String>, Option<Value>)> {
        let remote_viewer = match self.remote_viewer.as_ref() {
            Some(c) => c,
            None => bail!("please set RPC url at init argument"),
        };
        let output = remote_viewer.call_api(method.as_str(), params)?;
        Ok((None, Some(serde_json::to_value(&output)?)))
    }
}
fn panic_missing_public_key_named(cmd_name: &str, name: &IdentStr) -> ! {
    panic!(
        "Missing public key. Either add a `--public-key <priv_key>` argument \
            to the {} command, or associate an address to the \
            name '{}' in the init command.",
        cmd_name, name,
    )
}

fn panic_missing_public_key(cmd_name: &str) -> ! {
    panic!(
        "Missing public key. Try adding a `--public-key <priv_key>` \
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
        _test_path: &Path,
        default_syntax: SyntaxChoice,
        pre_compiled_deps: Option<&'a FullyCompiledProgram>,
        task_opt: Option<TaskInput<(InitCommand, Self::ExtraInitArgs)>>,
    ) -> Self {
        let (additional_mapping, extra_arg) = match task_opt.map(|t| t.command) {
            Some((InitCommand { named_addresses }, extra_arg)) => (
                verify_and_create_named_address_mapping(named_addresses).unwrap(),
                Some(extra_arg),
            ),
            None => (BTreeMap::new(), None),
        };

        // TODO: replace it with package's named address mapping.

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
        named_address_mapping.insert(
            "Std".to_string(),
            NumericalAddress::parse_str("0x1").unwrap(),
        );
        let mut public_key_mapping = BTreeMap::default();

        let init_args = extra_arg.unwrap_or_default();
        {
            // Public key mapping
            if let Some(additional_public_key_mapping) = init_args.public_keys {
                for (name, public_key) in additional_public_key_mapping {
                    if public_key_mapping.contains_key(&name) {
                        panic!(
                            "Invalid init. The named public key '{}' already exists.",
                            name
                        )
                    }
                    public_key_mapping.insert(name, public_key);
                }
            }
        }
        let mut remote_viewer: Option<RemoteViewer> = None;

        let store = if let Some(rpc) = init_args.rpc {
            let remote_view = RemoteViewer::from_url(&rpc, init_args.block_number).unwrap();
            remote_viewer = Some(remote_view.clone());
            SelectableStateView::B(InMemoryStateCache::new(remote_view))
        } else {
            let net = ChainNetwork::new_builtin(init_args.network.unwrap());
            let genesis_txn = Genesis::build_genesis_transaction(&net).unwrap();
            let data_store = ChainStateDB::mock();
            Genesis::execute_genesis_txn(&data_store, genesis_txn).unwrap();
            SelectableStateView::A(data_store)
        };

        // add pre compiled modules
        if let Some(pre_compiled_lib) = pre_compiled_deps {
            let mut writes = WriteSetMut::default();
            for c in &pre_compiled_lib.compiled {
                if let CompiledUnitEnum::Module(m) = c {
                    // update named_address_mapping
                    if let Some(named_address) = &m.address_name {
                        let name = named_address.value.to_string();
                        let already_assigned_with_different_value = named_address_mapping
                            .get(&name)
                            .filter(|existed| {
                                existed.into_inner() != m.named_module.address.into_inner()
                            })
                            .is_some();
                        if already_assigned_with_different_value {
                            panic!(
                                "Invalid init. The named address '{}' is already assigned with {}",
                                name,
                                named_address_mapping.get(&name).unwrap(),
                            )
                        }
                        named_address_mapping.insert(name, m.named_module.address);
                    }

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

        let genesis_key_pair = genesis_key_pair();

        let mut me = Self {
            compiled_state: CompiledState::new(named_address_mapping, pre_compiled_deps),
            default_syntax,
            public_key_mapping,
            storage: store,
            remote_viewer,
            debug: init_args.debug,
            association_public_key: genesis_key_pair.1.clone().into(),
        };
        me.hack_genesis_account()
            .expect("hack genesis account failure");

        me.public_key_mapping.insert(
            Identifier::new("Genesis").unwrap(),
            genesis_key_pair.1.clone().into(),
        );

        me.hack_account(association_address()).unwrap();
        me.public_key_mapping.insert(
            Identifier::new("StarcoinAssociation").unwrap(),
            genesis_key_pair.1.clone().into(),
        );
        // auto start from a new block based on existed state.
        me.handle_new_block(None, None, None, None)
            .expect("init test adapter failed");
        me
    }

    fn publish_module(
        &mut self,
        module: CompiledModule,
        named_addr_opt: Option<Identifier>,
        gas_budget: Option<u64>,
        extra: Self::ExtraPublishArgs,
    ) -> anyhow::Result<(Option<String>, Option<Value>)> {
        let module_id = module.self_id();
        let signer = module_id.address();
        let params = self.fetch_default_transaction_parameters(signer)?;

        let mut module_blob = vec![];
        module.serialize(&mut module_blob).unwrap();

        let public_key = match (extra.public_key, named_addr_opt) {
            (Some(key), _) => self.resolve_public_key(&key),
            (None, Some(named_addr)) => match self.public_key_mapping.get(&named_addr) {
                Some(key) => key.clone(),
                None => panic_missing_public_key_named("publish", &named_addr),
            },
            (None, None) => panic_missing_public_key("publish"),
        };

        let txn = RawUserTransaction::new_module(
            *signer,
            params.sequence_number,
            Module::new(module_blob),
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            params.chainid,
        );

        let output = self.run_transaction(txn, public_key)?;

        match output.output.status {
            TransactionStatusView::Executed => Ok((None, None)),
            _ => Ok((
                Some(format!("Publish failure: {:?}", output.output.status)),
                Some(serde_json::to_value(&output)?),
            )),
        }
    }

    fn execute_script(
        &mut self,
        script: CompiledScript,
        type_args: Vec<TypeTag>,
        signers: Vec<RawAddress>,
        args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
        extra_args: Self::ExtraRunArgs,
    ) -> anyhow::Result<(Option<String>, Option<Value>)> {
        assert!(!signers.is_empty());
        if signers.len() != 1 {
            panic!("Expected 1 signer, got {}.", signers.len());
        }
        let sender = self.compiled_state().resolve_address(&signers[0]);

        let mut script_blob = vec![];
        script.serialize(&mut script_blob)?;

        let params = self.fetch_default_transaction_parameters(&sender)?;

        let public_key = match (extra_args.public_key, &signers[0]) {
            (Some(public_key), _) => self.resolve_public_key(&public_key),
            (None, RawAddress::Named(named_addr)) => {
                match self.public_key_mapping.get(named_addr) {
                    Some(public_key) => public_key.clone(),
                    None => panic_missing_public_key_named("run", named_addr),
                }
            }
            (None, RawAddress::Anonymous(_)) => panic_missing_public_key("run"),
        };

        let txn = RawUserTransaction::new_script(
            sender,
            params.sequence_number,
            Script::new(script_blob, type_args, convert_txn_args(&args)),
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            params.chainid,
        );

        let output = self.run_transaction(txn, public_key)?;
        let result = SimpleTransactionResult {
            gas_used: output.output.gas_used.0,
            status: output.output.status.clone(),
        };
        Ok((
            Some(serde_json::to_string_pretty(&result)?),
            Some(serde_json::to_value(&output)?),
        ))
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
    ) -> anyhow::Result<(Option<String>, Option<Value>)> {
        {
            assert!(!signers.is_empty());
            if signers.len() != 1 {
                panic!("Expected 1 signer, got {}.", signers.len());
            }
        }
        let sender = self.compiled_state().resolve_address(&signers[0]);

        let params = self.fetch_default_transaction_parameters(&sender)?;

        let public_key = match (extra_args.public_key, &signers[0]) {
            (Some(public_key), _) => self.resolve_public_key(&public_key),
            (None, RawAddress::Named(named_addr)) => {
                match self.public_key_mapping.get(named_addr) {
                    Some(private_key) => private_key.clone(),
                    None => panic_missing_public_key_named("run", named_addr),
                }
            }
            (None, RawAddress::Anonymous(_)) => panic_missing_public_key("run"),
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
            params.chainid,
        );

        let output = self.run_transaction(txn, public_key)?;
        let result = SimpleTransactionResult {
            gas_used: output.output.gas_used.0,
            status: output.output.status.clone(),
        };
        Ok((
            Some(serde_json::to_string_pretty(&result)?),
            Some(serde_json::to_value(&output)?),
        ))
    }

    fn view_data(
        &mut self,
        address: AccountAddress,
        module: &ModuleId,
        resource: &IdentStr,
        type_args: Vec<TypeTag>,
    ) -> anyhow::Result<(String, Value)> {
        let s = RemoteStorage::new(&self.storage);
        view_resource_in_move_storage(&s, address, module, resource, type_args)
    }

    fn handle_subcommand(
        &mut self,
        subcommand: TaskInput<Self::Subcommand>,
    ) -> anyhow::Result<(Option<String>, Option<Value>)> {
        let (result_str, cmd_var_ctx) = match subcommand.command {
            StarcoinSubcommands::Faucet {
                address,
                initial_balance,
                public_key,
            } => self.handle_faucet(address, initial_balance, public_key),
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
            StarcoinSubcommands::CallAPI { method, params } => self.handle_call_api(method, params),
        }?;
        if self.debug {
            if let Some(cmd_var_ctx) = cmd_var_ctx.as_ref() {
                eprintln!("{}: {}", subcommand.name, cmd_var_ctx);
            }
        }
        Ok((result_str, cmd_var_ctx))
    }
}

/// Run the Starcoin transactional test flow, using the given file as input.
pub fn run_test(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_test_impl(path, Some(&*G_PRECOMPILED_STARCOIN_FRAMEWORK))
}

pub fn run_test_impl(
    path: &Path,
    fully_compiled_program_opt: Option<&FullyCompiledProgram>,
) -> Result<(), Box<dyn std::error::Error>> {
    framework::run_test_impl::<StarcoinTestAdapter>(path, fully_compiled_program_opt)
}
