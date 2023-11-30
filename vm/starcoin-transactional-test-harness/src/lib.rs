// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::context::ForkContext;
use anyhow::{bail, format_err, Result};
use clap::{Args, CommandFactory, Parser};
use move_binary_format::{file_format::CompiledScript, CompiledModule};
use move_command_line_common::address::ParsedAddress;
use move_command_line_common::files::verify_and_create_named_address_mapping;
use move_compiler::compiled_unit::{AnnotatedCompiledUnit, CompiledUnitEnum};
use move_compiler::shared::{NumberFormat, NumericalAddress, PackagePaths};
use move_compiler::{construct_pre_compiled_lib, FullyCompiledProgram};
use move_core_types::language_storage::StructTag;
use move_core_types::value::MoveValue;
use move_core_types::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, TypeTag},
};
use move_transactional_test_runner::framework;
use move_transactional_test_runner::tasks::{
    PrintBytecodeCommand, PublishCommand, RunCommand, ViewCommand,
};
use move_transactional_test_runner::{
    framework::{CompiledState, MoveTestAdapter},
    tasks::{InitCommand, SyntaxChoice, TaskInput},
    vm_test_harness::view_resource_in_move_storage,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use starcoin_abi_decoder::decode_txn_payload;
use starcoin_config::{genesis_key_pair, BuiltinNetworkID};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_dev::playground::call_contract;
use starcoin_rpc_api::types::{
    ContractCall, FunctionIdView, SignedUserTransactionView, TransactionArgumentView,
    TransactionOutputView, TransactionStatusView, TypeTagView,
};
use starcoin_rpc_api::Params;
use starcoin_state_api::{ChainStateReader, StateReaderExt};
use starcoin_types::account::{Account, AccountData};
use starcoin_types::block::{Block, BlockBody, BlockHeader, BlockHeaderExtra};
use starcoin_types::transaction::Package;
use starcoin_types::U256;
use starcoin_types::{
    access_path::AccessPath,
    account_config::{genesis_address, AccountResource},
    transaction::RawUserTransaction,
};
use starcoin_vm_runtime::session::SerializedReturnValues;
use starcoin_vm_runtime::{data_cache::RemoteStorage, starcoin_vm::StarcoinVM};
use starcoin_vm_types::account_config::{
    association_address, core_code_address, STC_TOKEN_CODE_STR,
};
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::authenticator::AccountPrivateKey;
use starcoin_vm_types::transaction::SignedUserTransaction;
use starcoin_vm_types::write_set::{WriteOp, WriteSetMut};
use starcoin_vm_types::{
    account_config::BalanceResource,
    block_metadata::BlockMetadata,
    genesis_config::ChainId,
    move_resource::MoveResource,
    on_chain_config::VMConfig,
    on_chain_resource,
    token::{stc::stc_type_tag, token_code::TokenCode},
    transaction::{Module, Script, ScriptFunction, Transaction, TransactionStatus},
    vm_status::KeptVMStatus,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::{collections::BTreeMap, convert::TryInto, path::Path, str::FromStr};
use stdlib::{starcoin_framework_named_addresses, stdlib_files};
use tempfile::{NamedTempFile, TempDir};

pub mod context;
pub mod fork_chain;
pub mod fork_state;
pub mod remote_state;

pub static G_FLAG_RELOAD_STDLIB: Mutex<bool> = Mutex::new(false);

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

    #[clap(
        long = "public-keys",
        takes_value(true),
        multiple_values(true),
        multiple_occurrences(true)
    )]
    /// the `public-keys` option is deprecated, please remove it.
    public_keys: Option<Vec<String>>,

    #[clap(long = "debug")]
    /// enable debug mode, output more info to stderr.
    debug: bool,
}

/// Starcoin-specific arguments for the publish command.
#[derive(Parser, Debug)]
pub struct StarcoinPublishArgs {}

/// Starcoin-specific arguments for the run command,
#[derive(Parser, Debug)]
pub struct StarcoinRunArgs {}

#[derive(Debug, Parser)]
#[clap(name = "faucet")]
struct FaucetSub {
    #[clap(long="addr", parse(try_from_str=ParsedAddress::parse))]
    /// faucet target address
    address: ParsedAddress,
    #[clap(long = "amount", default_value = "100000000000")]
    /// faucet amount
    initial_balance: u128,
}

#[derive(Debug, Parser)]
#[clap(name = "block")]
struct BlockSub {
    #[clap(long, parse(try_from_str=ParsedAddress::parse))]
    author: Option<ParsedAddress>,
    #[clap(long)]
    timestamp: Option<u64>,
    #[clap(long)]
    number: Option<u64>,
    #[clap(long)]
    uncles: Option<u64>,
}
#[derive(Debug, Parser)]
#[clap(name = "call")]
struct CallSub {
    #[clap(name = "FUNCTION")]
    /// smart contract function name.
    name: FunctionIdView,
    #[clap(long = "args", short = 'i')]

    /// function arguments.
    args: Vec<TransactionArgumentView>,

    #[clap(long = "type-args", short = 't')]
    /// function type arguments.
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

#[derive(Debug, Parser)]
#[clap(name = "package")]
pub struct PackageSub {
    #[clap(
    long = "signers",
    parse(try_from_str = ParsedAddress::parse),
    takes_value(true),
    multiple_values(true),
    multiple_occurrences(true)
    )]
    signers: Vec<ParsedAddress>,
    #[clap(long = "init-function")]
    /// module init function.
    init_function: Option<FunctionIdView>,
    #[clap(long = "type-args")]
    /// type arguments of init function.
    type_args: Option<Vec<TypeTagView>>,
    #[clap(long = "args")]
    /// arguments of init function.
    args: Option<Vec<TransactionArgumentView>>,
}

#[derive(Debug, Parser)]
#[clap(name = "deploy")]
pub struct DeploySub {
    #[clap(
    long = "signers",
    parse(try_from_str = ParsedAddress::parse),
    takes_value(true),
    multiple_values(true),
    multiple_occurrences(true)
    )]
    signers: Vec<ParsedAddress>,
    /// max gas for transaction.
    #[clap(long = "gas-budget")]
    gas_budget: Option<u64>,
    #[clap(name = "mv-or-package-file")]
    /// move bytecode file path or package binary path
    mv_or_package_file: PathBuf,
}

#[derive(Debug, Parser)]
#[clap(name = "var")]
pub struct VarSub {
    #[clap(name="var",
    parse(try_from_str = parse_var),
    takes_value(true),
    multiple_values(true),
    multiple_occurrences(true)
    )]
    /// variables with format <key1>=<value1>, <key2>=<value2>,...
    var: Vec<(String, String)>,
}

#[derive(Debug, Parser)]
#[clap(name = "read-json")]
pub struct ReadJsonSub {
    /// path of json file
    #[clap(name = "file")]
    file: PathBuf,
}

#[derive(Parser, Debug)]
pub enum StarcoinSubcommands {
    #[clap(name = "faucet")]
    Faucet {
        #[clap(long="addr", parse(try_from_str=ParsedAddress::parse))]
        address: ParsedAddress,
        #[clap(long = "amount", default_value = "100000000000")]
        initial_balance: u128,
    },

    #[clap(name = "block")]
    NewBlock {
        #[clap(long, parse(try_from_str=ParsedAddress::parse))]
        author: Option<ParsedAddress>,
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
    #[clap(name = "package")]
    Package {
        #[clap(long = "init-function")]
        init_function: Option<FunctionIdView>,
        #[clap(long = "type-args")]
        /// init function type args
        type_args: Option<Vec<TypeTagView>>,
        #[clap(long = "args")]
        /// init function args
        args: Option<Vec<TransactionArgumentView>>,
    },
    #[clap(name = "deploy")]
    Deploy {
        #[clap(
        long = "signers",
        parse(try_from_str = ParsedAddress::parse),
        takes_value(true),
        multiple_values(true),
        multiple_occurrences(true)
        )]
        signers: Vec<ParsedAddress>,
        #[clap(long = "gas-budget")]
        gas_budget: Option<u64>,
        #[clap(name = "mv-or-package-file")]
        /// move bytecode file path or package binary path
        mv_or_package_file: PathBuf,
    },
    #[clap(name = "var")]
    Var {
        #[clap(name="var",
        parse(try_from_str = parse_var),
        takes_value(true),
        multiple_values(true),
        multiple_occurrences(true)
        )]
        var: Vec<(String, String)>,
    },
    #[clap(name = "read-json")]
    ReadJson {
        /// max gas for transaction.
        #[clap(name = "file")]
        file: PathBuf,
    },
}

fn parse_params(params: &str) -> Result<Params> {
    let params = match params.trim() {
        "" => Params::None,
        param => serde_json::from_str(param)?,
    };
    Ok(params)
}

pub fn parse_var(s: &str) -> anyhow::Result<(String, String)> {
    let before_after = s.split('=').collect::<Vec<_>>();

    if before_after.len() != 2 {
        anyhow::bail!(
            "Invalid named var assignment. Must be of the form <key>=<value>, but \
             found '{}'",
            s
        );
    }
    let key = before_after[0].parse()?;
    let value = before_after[1].parse()?;
    Ok((key, value))
}

impl From<FaucetSub> for StarcoinSubcommands {
    fn from(sub: FaucetSub) -> Self {
        Self::Faucet {
            address: sub.address,
            initial_balance: sub.initial_balance,
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

impl From<PackageSub> for StarcoinSubcommands {
    fn from(sub: PackageSub) -> Self {
        Self::Package {
            init_function: sub.init_function,
            args: sub.args,
            type_args: sub.type_args,
        }
    }
}

impl From<DeploySub> for StarcoinSubcommands {
    fn from(sub: DeploySub) -> Self {
        Self::Deploy {
            signers: sub.signers,
            mv_or_package_file: sub.mv_or_package_file,
            gas_budget: sub.gas_budget,
        }
    }
}

impl From<ReadJsonSub> for StarcoinSubcommands {
    fn from(sub: ReadJsonSub) -> Self {
        Self::ReadJson { file: sub.file }
    }
}

impl From<VarSub> for StarcoinSubcommands {
    fn from(sub: VarSub) -> Self {
        Self::Var { var: sub.var }
    }
}

impl clap::Args for StarcoinSubcommands {
    fn augment_args(cmd: clap::Command<'_>) -> clap::Command<'_> {
        let faucet = FaucetSub::augment_args(clap::Command::new("faucet"));
        let block = BlockSub::augment_args(clap::Command::new("block"));
        let call = CallSub::augment_args(clap::Command::new("call"));
        let call_api = CallAPISub::augment_args(clap::Command::new("call-api"));
        let package = PackageSub::augment_args(clap::Command::new("package"));
        let deploy = DeploySub::augment_args(clap::Command::new("deploy"));
        let var = VarSub::augment_args(clap::Command::new("var"));
        let read_json = ReadJsonSub::augment_args(clap::Command::new("read-json"));
        cmd.subcommand(faucet)
            .subcommand(block)
            .subcommand(call)
            .subcommand(call_api)
            .subcommand(package)
            .subcommand(deploy)
            .subcommand(var)
            .subcommand(read_json)
    }

    fn augment_args_for_update(_cmd: clap::Command<'_>) -> clap::Command<'_> {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransactionWithOutput {
    pub txn: SignedUserTransactionView,
    pub output: TransactionOutputView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimpleTransactionResult {
    gas_used: u64,
    status: TransactionStatusView,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct PackageResult {
    pub file: String,
    pub package_hash: HashValue,
    pub hex: String,
}

pub struct StarcoinTestAdapter<'a> {
    compiled_state: CompiledState<'a>,
    // storage: SelectableStateView<ChainStateDB, InMemoryStateCache<RemoteViewer>>,
    default_syntax: SyntaxChoice,
    context: ForkContext,
    tempdir: TempDir,
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
    fn sign(&self, raw_txn: RawUserTransaction) -> SignedUserTransaction {
        let keypair = genesis_key_pair();
        let account_private_key: AccountPrivateKey = keypair.0.into();
        let auth = account_private_key.sign(&raw_txn);
        SignedUserTransaction::new(raw_txn, auth)
    }

    /// Obtain a Rust representation of the account resource from storage, which is used to derive
    /// a few default transaction parameters.
    fn fetch_account_resource(&self, signer_addr: &AccountAddress) -> Result<AccountResource> {
        let account_access_path =
            AccessPath::resource_access_path(*signer_addr, AccountResource::struct_tag());
        let account_blob = self
            .context
            .storage
            .get_state_value(&StateKey::AccessPath(account_access_path))?
            .ok_or_else(|| {
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

        let balance_blob = self
            .context
            .storage
            .get_state_value(&StateKey::AccessPath(balance_access_path))?
            .ok_or_else(|| {
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
        self.context
            .apply_write_set(genesis_account_data.to_writeset())?;

        {
            let mut writes = WriteSetMut::default();
            writes.push((
                StateKey::AccessPath(AccessPath::resource_access_path(
                    genesis_address(),
                    StructTag {
                        address: genesis_address(),
                        module: Identifier::new("Account")?,
                        name: Identifier::new("SignerDelegated")?,
                        type_params: vec![],
                    },
                )),
                WriteOp::Deletion,
            ));
            self.context.apply_write_set(writes.freeze().unwrap())?;
        }
        Ok(())
    }

    /// Hack the account, and set account's auth key to genesis keypair
    fn hack_account(&self, address: AccountAddress) -> Result<()> {
        if self.debug {
            eprintln!("Hack account {}", address);
        }
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
        self.context.apply_write_set(account_data.to_writeset())?;
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
            .context
            .storage
            .get_on_chain_config::<VMConfig>()?
            .ok_or_else(|| anyhow::anyhow!("Failed to fetch onchain vm config."))?;
        let max_number_of_gas_units = vmconfig
            .gas_schedule
            .gas_constants
            .maximum_number_of_gas_units;
        let gas_unit_price = 1;
        let max_gas_amount = if gas_unit_price == 0 {
            max_number_of_gas_units
        } else {
            let account_balance =
                self.fetch_balance_resource(signer_addr, stc_type_tag().to_string())?;
            std::cmp::min(
                max_number_of_gas_units,
                (account_balance.token() / gas_unit_price as u128) as u64,
            )
        };
        let chain_id = self.context.storage.get_chain_id()?;
        Ok(TransactionParameters {
            sequence_number,
            gas_unit_price,
            max_gas_amount,
            expiration_timestamp_secs: self.context.storage.get_timestamp()?.seconds() + 60 * 60,
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
            &self.context.storage,
            vec![Transaction::BlockMetadata(meta.clone())],
            None,
        )?;
        assert_eq!(outputs.len(), 1);

        let (status, output) = outputs.pop().unwrap();
        match output.status() {
            TransactionStatus::Keep(kept_vm_status) => match kept_vm_status {
                KeptVMStatus::Executed => {
                    self.context
                        .apply_write_set(output.clone().into_inner().1)?;
                }
                _ => {
                    bail!("Failed to execute transaction. VMStatus: {}", status)
                }
            },
            TransactionStatus::Discard(_) => {
                bail!("Transaction discarded. VMStatus: {}", status)
            }
            TransactionStatus::Retry => {
                bail!("Transaction Retry never happen")
            }
        }
        let mut chain = self.context.chain.lock().unwrap();
        chain.add_new_txn(Transaction::BlockMetadata(meta), output)?;

        Ok(())
    }

    /// Perform a single Starcoin transaction.
    ///
    /// Should error if the transaction ends up being discarded, or having a status other than
    /// EXECUTED.
    fn run_transaction(&mut self, txn: RawUserTransaction) -> Result<TransactionWithOutput> {
        let mut vm = StarcoinVM::new(None);
        let signed_txn = self.sign(txn);

        let (_status, output) = vm
            .execute_block_transactions(
                &self.context.storage,
                vec![Transaction::UserTransaction(signed_txn.clone())],
                None,
            )?
            .pop()
            .unwrap();
        match output.status() {
            TransactionStatus::Keep(_kept_vm_status) => {
                self.context
                    .apply_write_set(output.clone().into_inner().1)?;
                let mut chain = self.context.chain.lock().unwrap();
                chain.add_new_txn(
                    Transaction::UserTransaction(signed_txn.clone()),
                    output.clone(),
                )?;
            }
            TransactionStatus::Discard(_) => {}
            TransactionStatus::Retry => {}
        }
        let payload = decode_txn_payload(&self.context.storage, signed_txn.payload())?;
        let mut txn_view: SignedUserTransactionView = signed_txn.try_into()?;
        txn_view.raw_txn.decoded_payload = Some(payload.into());
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
            &self.context.storage,
            function_id.0.module,
            function_id.0.function.as_str(),
            type_args.into_iter().map(|t| t.0).collect(),
            args.into_iter().map(|t| t.0).collect(),
            None,
        )?;

        let move_resolver = RemoteStorage::new(&self.context.storage);
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
        addr: ParsedAddress,
        initial_balance: u128,
    ) -> Result<(Option<String>, Option<Value>)> {
        let sender = association_address();

        let params = self.fetch_default_transaction_parameters(&sender)?;

        match &addr {
            ParsedAddress::Named(name) => {
                if !self
                    .compiled_state
                    .named_address_mapping
                    .contains_key(name.as_str())
                {
                    // make it deterministic.

                    let addr = AccountAddress::from_bytes(
                        &HashValue::sha3_256_of(name.as_bytes()).as_slice()
                            [0..AccountAddress::LENGTH],
                    )?;

                    self.compiled_state.named_address_mapping.insert(
                        name.clone(),
                        NumericalAddress::new(addr.into_bytes(), NumberFormat::Hex),
                    );
                }
            }
            ParsedAddress::Numerical(_addr) => {}
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
        let output = self.run_transaction(txn)?;

        match output.output.status {
            TransactionStatusView::Executed => {
                self.hack_account(addr)?;
                Ok((None, Some(serde_json::to_value(&output)?)))
            }
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
        author: Option<ParsedAddress>,
        timestamp: Option<u64>,
        number: Option<u64>,
        uncles: Option<u64>,
    ) -> Result<(Option<String>, Option<Value>)> {
        let last_blockmeta = self
            .context
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
        let timestamp =
            timestamp.unwrap_or(self.context.storage.get_timestamp()?.milliseconds + 10 * 1000);
        //TODO find a better way to get parent hash, we should keep to local storage.
        let parent_hash = self.context.chain.lock().unwrap().head_block_hash();

        let new_block_meta = BlockMetadata::new(
            parent_hash,
            timestamp,
            author,
            None,
            uncles,
            height,
            self.context.storage.get_chain_id()?,
            0,
        );
        self.run_blockmeta(new_block_meta.clone()).map_err(|e| {
            println!("Run blockmeta error: {}", e);
            e
        })?;

        let (parent_hash, timestamp, author, _author_auth_key, _, number, _, _) =
            new_block_meta.clone().into_inner();
        let block_body = BlockBody::new(vec![], None);
        let block_header = BlockHeader::new(
            parent_hash,
            timestamp,
            number,
            author,
            self.context.chain.lock().unwrap().txn_accumulator_root(),
            HashValue::random(),
            self.context.storage.state_root(),
            0u64,
            U256::zero(),
            block_body.hash(),
            self.context.storage.get_chain_id()?,
            0,
            BlockHeaderExtra::new([0u8; 4]),
            None,
        );
        let new_block = Block::new(block_header, block_body);
        let mut chain = self.context.chain.lock().unwrap();
        chain.add_new_block(new_block)?;

        Ok((None, Some(serde_json::to_value(&new_block_meta)?)))
    }

    fn handle_call_api(
        &mut self,
        method: String,
        params: Params,
    ) -> Result<(Option<String>, Option<Value>)> {
        let output = self.context.call_api(method.as_str(), params)?;
        Ok((None, Some(serde_json::to_value(output)?)))
    }

    fn build_package(
        modules: Vec<CompiledModule>,
        init_function: Option<ScriptFunction>,
    ) -> Result<Package> {
        let mut ms = vec![];
        for m in modules {
            let mut code = vec![];
            m.serialize(&mut code)?;
            ms.push(Module::new(code));
        }
        Package::new(ms, init_function)
    }

    fn handle_package(
        &mut self,
        data: Option<NamedTempFile>,
        init_function: Option<FunctionIdView>,
        type_args: Option<Vec<TypeTagView>>,
        args: Option<Vec<TransactionArgumentView>>,
    ) -> Result<(Option<String>, Option<Value>)> {
        let data = match data {
            Some(f) => f,
            None => panic!("Expected a module text block following 'package'",),
        };
        let data_path = data.path().to_str().unwrap();
        let (named_addr_opt, module, warnings_opt) = {
            let (unit, warnings_opt) = self.compiled_state.complie(data_path)?;
            match unit {
                AnnotatedCompiledUnit::Module(annot_module) => {
                    let (named_addr_opt, _id) = annot_module.module_id();
                    (
                        named_addr_opt.map(|n| n.value),
                        annot_module.named_module.module,
                        warnings_opt,
                    )
                }
                AnnotatedCompiledUnit::Script(_) => {
                    panic!("Expected a module text block, not a script, following 'package'")
                }
            }
        };

        self.compiled_state
            .add_precompiled(named_addr_opt, module.clone());

        let package = Self::build_package(
            vec![module.clone()],
            init_function.map(|fid| {
                let move_args = &args
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| v.0)
                    .collect::<Vec<_>>();
                let move_args = move_args
                    .iter()
                    .map(|arg| MoveValue::from(arg.clone()))
                    .collect::<Vec<_>>();
                ScriptFunction::new(
                    fid.0.module,
                    fid.0.function,
                    type_args
                        .unwrap_or_default()
                        .into_iter()
                        .map(|v| v.0)
                        .collect(),
                    convert_txn_args(&move_args),
                )
            }),
        )?;

        let package_hash = package.crypto_hash();
        let output_file = {
            let mut output_file = self.tempdir.path().join(package_hash.to_string());
            output_file.set_extension("blob");
            output_file
        };
        let mut file = File::create(output_file.as_path())?;
        let blob = bcs_ext::to_bytes(&package)?;
        let hex = format!("0x{}", hex::encode(blob.as_slice()));
        file.write_all(&blob)
            .map_err(|e| format_err!("write package file {:?} error:{:?}", output_file, e))?;

        let package_result = PackageResult {
            file: output_file.to_str().unwrap().to_string(),
            package_hash,
            hex,
        };
        self.compiled_state().add_with_source_file(
            named_addr_opt,
            module,
            (data_path.to_owned(), data),
        );
        Ok((warnings_opt, Some(serde_json::to_value(&package_result)?)))
    }

    fn handle_deploy(
        &mut self,
        signers: Vec<ParsedAddress>,
        gas_budget: Option<u64>,
        package_path: &Path,
    ) -> Result<(Option<String>, Option<Value>)> {
        let mut bytes = vec![];
        File::open(package_path)?.read_to_end(&mut bytes)?;

        let package: Package = bcs_ext::from_bytes(&bytes).map_err(|e| {
            format_err!(
                "Decode Package failed {:?}, please ensure the file is a Package binary file.",
                e
            )
        })?;

        let signer = match signers.get(0) {
            Some(addr) => self.compiled_state.resolve_address(addr),
            None => package.package_address(),
        };
        let params = self.fetch_default_transaction_parameters(&signer)?;

        let txn = RawUserTransaction::new_package(
            signer,
            params.sequence_number,
            package,
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            params.chainid,
        );

        let output = self.run_transaction(txn)?;

        match output.output.status {
            TransactionStatusView::Executed => Ok((None, None)),
            _ => Ok((
                Some(format!("Publish failure: {:?}", output.output.status)),
                Some(serde_json::to_value(&output)?),
            )),
        }
    }

    fn handle_var(
        &mut self,
        vars: Vec<(String, String)>,
    ) -> Result<(Option<String>, Option<Value>)> {
        let var_dict: HashMap<String, String> = vars.into_iter().collect();
        Ok((None, Some(serde_json::to_value(var_dict)?)))
    }

    fn handle_read_json(&mut self, file: &Path) -> Result<(Option<String>, Option<Value>)> {
        let content: serde_json::Value = serde_json::from_reader(File::open(file)?)?;
        Ok((None, Some(content)))
    }
}

impl<'a> MoveTestAdapter<'a> for StarcoinTestAdapter<'a> {
    type ExtraPublishArgs = StarcoinPublishArgs;
    type ExtraValueArgs = ();
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
    ) -> (Self, Option<String>) {
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

        let init_args = extra_arg.unwrap_or_default();

        if init_args.public_keys.is_some() {
            eprintln!("[WARN] the `public_keys` option is deprecated, and is no longer working, please remove it.");
        }

        let (context, fork_flag) = if let Some(rpc) = init_args.rpc {
            (
                ForkContext::new_fork(&rpc, init_args.block_number).unwrap(),
                true,
            )
        } else {
            let stdlib_modules = if *G_FLAG_RELOAD_STDLIB.lock().unwrap() {
                assert!(
                    pre_compiled_deps.is_some(),
                    "Current project must be framework."
                );
                let mut modules: Vec<Vec<u8>> = vec![];
                for c in &pre_compiled_deps.unwrap().compiled {
                    if let CompiledUnitEnum::Module(m) = c {
                        let mut buffer: Vec<u8> = vec![];
                        m.named_module.module.serialize(&mut buffer).unwrap();
                        modules.push(buffer);
                    }
                }
                Some(modules)
            } else {
                None
            };
            (
                ForkContext::new_local(init_args.network.unwrap(), stdlib_modules).unwrap(),
                false,
            )
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
                        StateKey::AccessPath(AccessPath::code_access_path(
                            m.named_module.address.into_inner(),
                            Identifier::new(m.named_module.name.as_str()).unwrap(),
                        )),
                        WriteOp::Value({
                            let mut bytes = vec![];
                            m.named_module.module.serialize(&mut bytes).unwrap();
                            bytes
                        }),
                    ));
                }
            }
            context.apply_write_set(writes.freeze().unwrap()).unwrap();
        }

        let mut me = Self {
            compiled_state: CompiledState::new(named_address_mapping, pre_compiled_deps, None),
            default_syntax,
            context,
            tempdir: TempDir::new().unwrap(),
            debug: init_args.debug,
        };
        me.hack_genesis_account()
            .expect("hack genesis account failure");

        me.hack_account(association_address()).unwrap();

        if fork_flag {
        } else {
            // auto start from a new block based on existed state.
            me.handle_new_block(None, None, None, None)
                .expect("init test adapter failed");
        };
        (me, None)
    }

    fn publish_module(
        &mut self,
        module: CompiledModule,
        named_addr_opt: Option<Identifier>,
        gas_budget: Option<u64>,
        _extra: Self::ExtraPublishArgs,
    ) -> anyhow::Result<(Option<String>, CompiledModule, Option<Value>)> {
        let module_id = module.self_id();
        let signer = match named_addr_opt {
            Some(name) => self.compiled_state.resolve_named_address(name.as_str()),
            None => *module_id.address(),
        };
        let params = self.fetch_default_transaction_parameters(&signer)?;

        let package = Self::build_package(vec![module.clone()], None)?;
        let txn = RawUserTransaction::new_package(
            signer,
            params.sequence_number,
            package,
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            params.chainid,
        );

        let output = self.run_transaction(txn)?;

        match output.output.status {
            TransactionStatusView::Executed => Ok((None, module, None)),
            _ => Ok((
                Some(format!("Publish failure: {:?}", output.output.status)),
                module,
                Some(serde_json::to_value(&output)?),
            )),
        }
    }

    fn execute_script(
        &mut self,
        script: CompiledScript,
        type_args: Vec<TypeTag>,
        signers: Vec<ParsedAddress>,
        args: Vec<MoveValue>,
        gas_budget: Option<u64>,
        _extra_args: Self::ExtraRunArgs,
    ) -> anyhow::Result<(Option<String>, SerializedReturnValues, Option<Value>)> {
        assert!(!signers.is_empty());
        if signers.len() != 1 {
            panic!("Expected 1 signer, got {}.", signers.len());
        }
        let sender = self.compiled_state().resolve_address(&signers[0]);

        let mut script_blob = vec![];
        script.serialize(&mut script_blob)?;

        let params = self.fetch_default_transaction_parameters(&sender)?;

        // orignal 0xc867 become 0x0c383637, this convert 0x0c383637 => 0xc867
        let mut args_vec = vec![];
        for arg in args.into_iter() {
            match arg {
                MoveValue::Vector(vals) => {
                    let mut is_vec_u8 = true;
                    for val in vals.iter() {
                        match val {
                            MoveValue::U8(_) => {}
                            _ => is_vec_u8 = false,
                        }
                    }
                    if vals.len() % 2 == 1 || vals.is_empty() {
                        is_vec_u8 = false;
                    }
                    match is_vec_u8 {
                        true => {
                            assert_eq!(vals.get(0), Some(&MoveValue::U8(48)));
                            assert_eq!(vals.get(1), Some(&MoveValue::U8(120)));
                            let mut vals_compress = vec![];
                            for i in (2..vals.len()).step_by(2) {
                                let x = vals.get(i).cloned();
                                let y = vals.get(i + 1).cloned();
                                match (x, y) {
                                    (Some(MoveValue::U8(a)), Some(MoveValue::U8(b))) => {
                                        let val = (convert_u8(a) << 4) | convert_u8(b);
                                        vals_compress.push(MoveValue::U8(val));
                                    }
                                    _ => panic!("is not possible"),
                                }
                            }
                            args_vec.push(MoveValue::Vector(vals_compress));
                        }
                        false => args_vec.push(MoveValue::Vector(vals)),
                    }
                }
                _ => args_vec.push(arg),
            }
        }
        let txn = RawUserTransaction::new_script(
            sender,
            params.sequence_number,
            Script::new(script_blob, type_args, convert_txn_args(&args_vec)),
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            params.chainid,
        );

        let output = self.run_transaction(txn)?;
        let result = SimpleTransactionResult {
            gas_used: output.output.gas_used.0,
            status: output.output.status.clone(),
        };
        let value = SerializedReturnValues {
            mutable_reference_outputs: vec![],
            return_values: vec![],
        };
        Ok((
            Some(serde_json::to_string_pretty(&result)?),
            value,
            Some(serde_json::to_value(&output)?),
        ))
    }

    fn call_function(
        &mut self,
        module: &ModuleId,
        function: &IdentStr,
        type_args: Vec<TypeTag>,
        signers: Vec<ParsedAddress>,
        args: Vec<MoveValue>,
        gas_budget: Option<u64>,
        _extra_args: Self::ExtraRunArgs,
    ) -> anyhow::Result<(Option<String>, SerializedReturnValues, Option<Value>)> {
        {
            assert!(!signers.is_empty());
            if signers.len() != 1 {
                panic!("Expected 1 signer, got {}.", signers.len());
            }
        }
        let sender = self.compiled_state().resolve_address(&signers[0]);

        let params = self.fetch_default_transaction_parameters(&sender)?;

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

        let output = self.run_transaction(txn)?;
        let result = SimpleTransactionResult {
            gas_used: output.output.gas_used.0,
            status: output.output.status.clone(),
        };
        let value = SerializedReturnValues {
            mutable_reference_outputs: vec![],
            return_values: vec![],
        };
        Ok((
            Some(serde_json::to_string_pretty(&result)?),
            value,
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
        let s = RemoteStorage::new(&self.context.storage);
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
            } => self.handle_faucet(address, initial_balance),
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
            StarcoinSubcommands::Package {
                init_function,
                type_args,
                args,
            } => self.handle_package(subcommand.data, init_function, type_args, args),
            StarcoinSubcommands::Deploy {
                signers,
                gas_budget,
                mv_or_package_file,
            } => self.handle_deploy(signers, gas_budget, mv_or_package_file.as_path()),
            StarcoinSubcommands::Var { var } => self.handle_var(var),
            StarcoinSubcommands::ReadJson { file } => self.handle_read_json(file.as_path()),
        }?;
        if self.debug {
            if let Some(cmd_var_ctx) = cmd_var_ctx.as_ref() {
                eprintln!("{}: {}", subcommand.name, cmd_var_ctx);
            }
        }
        Ok((result_str, cmd_var_ctx))
    }
}

fn convert_txn_args(args: &[MoveValue]) -> Vec<Vec<u8>> {
    args.iter()
        .map(|arg| {
            arg.simple_serialize()
                .expect("transaction arguments must serialize")
        })
        .collect()
}

/// Run the Starcoin transactional test flow, using the given file as input.
pub fn run_test(path: &Path) -> Result<(), Box<dyn std::error::Error + 'static>> {
    run_test_impl(path, Some(&*G_PRECOMPILED_STARCOIN_FRAMEWORK))
}

pub fn run_test_impl<'a>(
    path: &Path,
    fully_compiled_program_opt: Option<&'a FullyCompiledProgram>,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    framework::run_test_impl::<StarcoinTestAdapter>(path, fully_compiled_program_opt)
}

pub fn print_help(task_name: Option<String>) -> Result<()> {
    let mut tasks = HashMap::new();
    tasks.insert(
        "init",
        ExtraInitArgs::augment_args(InitCommand::command().name("init")),
    );
    tasks.insert(
        "print-bytecode",
        PrintBytecodeCommand::command().name("print-bytecode"),
    );
    tasks.insert("publish", PublishCommand::command().name("publish"));
    tasks.insert("run", RunCommand::<()>::command().name("run"));
    tasks.insert("view", ViewCommand::command().name("view"));
    tasks.insert("faucet", FaucetSub::command().name("faucet"));
    tasks.insert("block", BlockSub::command().name("block"));
    tasks.insert("call", CallSub::command().name("call"));
    tasks.insert("call-api", CallAPISub::command().name("call-api"));
    tasks.insert("package", PackageSub::command().name("package"));
    tasks.insert("deploy", DeploySub::command().name("deploy"));
    tasks.insert("var", VarSub::command().name("var"));
    tasks.insert("read-json", ReadJsonSub::command().name("read-json"));

    match task_name {
        Some(name) => match tasks.get_mut(&name[..]) {
            Some(cmd) => cmd
                .print_help()
                .map_err(|e| format_err!("print help error: {:?}", e)),
            None => bail!("Task {:?} not found.", name),
        },
        None => {
            {
                for cmd in tasks.values_mut() {
                    println!("------------------------------------------------");
                    cmd.print_help()?;
                    println!();
                }
            };
            Ok(())
        }
    }
}

pub static G_PRECOMPILED_STARCOIN_FRAMEWORK: Lazy<FullyCompiledProgram> = Lazy::new(|| {
    let sources = stdlib_files();
    let program_res = construct_pre_compiled_lib(
        vec![PackagePaths {
            name: None,
            paths: sources,
            named_address_map: starcoin_framework_named_addresses(),
        }],
        None,
        move_compiler::Flags::empty(),
    )
    .unwrap();
    match program_res {
        Ok(df) => df,
        Err((files, errors)) => {
            eprintln!("!!!Starcoin Framework failed to compile!!!");
            move_compiler::diagnostics::report_diagnostics(&files, errors)
        }
    }
});

fn convert_u8(val: u8) -> u8 {
    // val >= 'a' && val <= 'z'
    if (97..=122).contains(&val) {
        val - 97 + 10
    } else {
        // val >= '0' && val <= '9'
        val - 48
    }
}
