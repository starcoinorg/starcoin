// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, Result};
use move_cli::dependencies::ModuleDependencyResolver;
use move_cli::function_resolver::FunctionResolver;
use move_cli::package::DepMode;
use move_cli::remote_state::{resolve_deps, MergedRemoteCache, RemoteStateView};
use move_cli::{
    package::{parse_mode_from_string, Mode},
    *,
};
use move_command_line_common::files::MOVE_COMPILED_EXTENSION;
use move_core_types::errmap::ErrorMapping;
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Event},
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    parser,
    transaction_argument::{convert_txn_args, TransactionArgument},
    vm_status::{AbortLocation, StatusCode, VMStatus},
};
use move_lang::shared::Flags;
use move_lang::{self, compiled_unit::CompiledUnit, Compiler, MOVE_COMPILED_INTERFACES_DIR};
use move_unit_test::UnitTestingConfig;
use move_vm_runtime::data_cache::MoveStorage;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_runtime::move_vm_adapter::SessionAdapter;
use move_vm_runtime::session::Session;
use starcoin_config::INITIAL_GAS_SCHEDULE;
use starcoin_functional_tests::executor::FakeExecutor;
use starcoin_functional_tests::testsuite::PRETTY;
use starcoin_vm_runtime::natives::starcoin_natives;
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::gas_schedule::GasStatus;
use std::num::NonZeroUsize;
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use vm::errors::Location;
use vm::normalized::Module;
use vm::{
    access::ModuleAccess,
    compatibility::Compatibility,
    errors::{PartialVMError, VMError},
    file_format::{AbilitySet, CompiledModule, CompiledScript, SignatureToken},
    normalized,
};

#[derive(StructOpt)]
#[structopt(
    name = "move",
    about = "CLI frontend for Move compiler and VM",
    rename_all = "kebab-case"
)]
pub struct Move {
    /// Directory storing Move resources, events, and module bytecodes produced by module publishing
    /// and script execution.
    #[structopt(long, default_value = DEFAULT_STORAGE_DIR, global = true)]
    storage_dir: String,
    /// Directory storing build artifacts produced by compilation
    #[structopt(long, short = "d", default_value = DEFAULT_BUILD_DIR, global = true)]
    build_dir: String,
    /// Dependency inclusion mode
    #[structopt(
        long,
        default_value = DEFAULT_DEP_MODE,
        global = true,
        parse(try_from_str = parse_mode_from_string),
    )]
    mode: Mode,
    /// starcoin rpc, required if dep mode is starcoin
    #[structopt(
        long,
        default_value = "http://main.seed.starcoin.org:9850",
        global = true,
        required_if("mode", "starcoin")
    )]
    starcoin_rpc: String,
    #[structopt(long, global = true)]
    /// block height to fork from. default to latest block number
    block_number: Option<u64>,
    /// Print additional diagnostics
    #[structopt(short = "v", global = true)]
    verbose: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
    /// Create a scaffold for developing.
    #[structopt(name = "scaffold")]
    Scaffold {
        /// a directory path where scaffold will be saved
        #[structopt(name = "path")]
        path: String,
    },
    /// Type check and verify the specified script and modules against the modules in `storage`
    #[structopt(name = "check")]
    Check {
        /// The source files to check
        #[structopt(
            name = "PATH_TO_SOURCE_FILE",
            default_value = DEFAULT_SOURCE_DIR,
        )]
        source_files: Vec<String>,
        /// If set, fail when attempting to typecheck a module that already exists in global storage
        #[structopt(long = "no-republish")]
        no_republish: bool,
    },
    /// Compile the specified modules and publish the resulting bytecodes in global storage
    #[structopt(name = "publish")]
    Publish {
        /// The source files containing modules to publish
        #[structopt(
            name = "PATH_TO_SOURCE_FILE",
            default_value = DEFAULT_SOURCE_DIR,
        )]
        source_files: Vec<String>,
        /// If set, fail during compilation when attempting to publish a module that already
        /// exists in global storage
        #[structopt(long = "no-republish")]
        no_republish: bool,
        /// By default, code that might cause breaking changes for bytecode
        /// linking or data layout compatibility checks will not be published.
        /// Set this flag to ignore breaking changes checks and publish anyway
        #[structopt(long = "ignore-breaking-changes")]
        ignore_breaking_changes: bool,
    },
    /// Compile/run a Move script that reads/writes resources stored on disk in `storage`.
    /// This command compiles the script first before running it.
    #[structopt(name = "run")]
    Run {
        /// Path to .mv file containing either script or module bytecodes. If the file is a module, the
        /// `script_name` parameter must be set.
        #[structopt(name = "script")]
        script_file: String,
        /// Name of the script function inside `script_file` to call. Should only be set if `script_file`
        /// points to a module.
        #[structopt(name = "name")]
        script_name: Option<String>,
        /// Possibly-empty list of signers for the current transaction (e.g., `account` in
        /// `main(&account: signer)`). Must match the number of signers expected by `script_file`.
        #[structopt(long = "signers")]
        signers: Vec<String>,
        /// Possibly-empty list of arguments passed to the transaction (e.g., `i` in
        /// `main(i: u64)`). Must match the arguments types expected by `script_file`.
        /// Supported argument types are
        /// bool literals (true, false),
        /// u64 literals (e.g., 10, 58),
        /// address literals (e.g., 0x12, 0x0000000000000000000000000000000f),
        /// hexadecimal strings (e.g., x"0012" will parse as the vector<u8> value [00, 12]), and
        /// ASCII strings (e.g., 'b"hi" will parse as the vector<u8> value [68, 69])
        #[structopt(long = "args", parse(try_from_str = parser::parse_transaction_argument))]
        args: Vec<TransactionArgument>,
        /// Possibly-empty list of type arguments passed to the transaction (e.g., `T` in
        /// `main<T>()`). Must match the type arguments kinds expected by `script_file`.
        #[structopt(long = "type-args", parse(try_from_str = parser::parse_type_tag))]
        type_args: Vec<TypeTag>,
        /// Maximum number of gas units to be consumed by execution.
        /// When the budget is exhaused, execution will abort.
        /// By default, no `gas-budget` is specified and gas metering is disabled.
        #[structopt(long = "gas-budget", short = "g")]
        gas_budget: Option<u64>,
        /// If set, the effects of executing `script_file` (i.e., published, updated, and
        /// deleted resources) will NOT be committed to disk.
        #[structopt(long = "dry-run", short = "n")]
        dry_run: bool,
    },

    /// run functional test under tests dir.
    #[structopt(name = "functional-test")]
    FunctionalTest {
        /// The FILTER string is tested against the name of all tests, and only those tests whose names
        /// contain the filter are run.
        filter: Option<String>,
        #[structopt(long = "exact")]
        /// Exactly match filters rather than by substring
        filter_exact: bool,
        #[structopt(long, default_value = "32", env = "RUST_TEST_THREADS")]
        /// Number of threads used for running tests in parallel
        test_threads: NonZeroUsize,
        #[structopt(short = "q", long)]
        /// Output minimal information
        quiet: bool,
        #[structopt(long)]
        /// List all tests
        list: bool,
        #[structopt(long)]
        /// List or run ignored tests (always empty: it is currently not possible to mark tests as
        /// ignored)
        ignored: bool,
    },

    /// Run unit test in move source files.
    #[structopt(name = "unit-test")]
    UnitTest {
        /// Bound the number of instructions that can be executed by any one test.
        #[structopt(
            name = "instructions",
            default_value = "5000",
            short = "i",
            long = "instructions"
        )]
        instruction_execution_bound: u64,

        /// A filter string to determine which unit tests to run
        #[structopt(name = "filter", short = "f", long = "filter")]
        filter: Option<String>,

        /// List all tests
        #[structopt(name = "list", short = "l", long = "list")]
        list: bool,

        /// Number of threads to use for running tests.
        #[structopt(
            name = "num_threads",
            default_value = "8",
            short = "t",
            long = "threads"
        )]
        num_threads: usize,

        /// Report test statistics at the end of testing
        #[structopt(name = "report_statistics", short = "s", long = "statistics")]
        report_statistics: bool,

        /// Show the storage state at the end of execution of a failing test
        #[structopt(name = "global_state_on_error", short = "g", long = "state_on_error")]
        report_storage_on_error: bool,

        /// The source files
        #[structopt(
        name = "PATH_TO_SOURCE_FILE",
        default_value = DEFAULT_SOURCE_DIR,
        )]
        source_files: Vec<String>,

        /// Use the stackless bytecode interpreter to run the tests and cross check its results with
        /// the execution result from Move VM.
        #[structopt(long = "stackless")]
        check_stackless_vm: bool,
    },

    /// Run expected value tests using the given batch file
    #[structopt(name = "test")]
    Test {
        /// a directory path in which all the tests will be executed
        #[structopt(name = "path")]
        path: String,
        /// Show coverage information after tests are done.
        /// By default, coverage will not be tracked nor shown.
        #[structopt(long = "track-cov")]
        track_cov: bool,
        /// Create a new test directory scaffold with the specified <path>
        #[structopt(long = "create")]
        create: bool,
    },
    /// View Move resources, events files, and modules stored on disk
    #[structopt(name = "view")]
    View {
        /// Path to a resource, events file, or module stored on disk.
        #[structopt(name = "file")]
        file: String,
    },
    /// Delete all resources, events, and modules stored on disk under `storage`.
    /// Does *not* delete anything in `src`.
    Clean {},
    /// Run well-formedness checks on the `storage` and `build` directories.
    #[structopt(name = "doctor")]
    Doctor {},
}

impl Move {
    fn get_package_dir(&self) -> PathBuf {
        Path::new(&self.build_dir).join(DEFAULT_PACKAGE_DIR)
    }

    /// This collects only the compiled modules from dependent libraries. The modules
    /// created via the "publish" command should already sit in the storage based on
    /// current implementation.
    fn get_library_modules(&self) -> Result<Vec<CompiledModule>> {
        self.mode.compiled_modules(&self.get_package_dir())
    }

    /// Return `true` if `path` is a Move bytecode file based on its extension
    fn is_bytecode_file(path: &Path) -> bool {
        path.extension()
            .map_or(false, |ext| ext == MOVE_COMPILED_EXTENSION)
    }

    /// Return `true` if path contains a valid Move bytecode module
    fn contains_module(path: &Path) -> bool {
        Self::is_bytecode_file(path)
            && match fs::read(path) {
                Ok(bytes) => CompiledModule::deserialize(&bytes).is_ok(),
                Err(_) => false,
            }
    }

    /// Prepare an OnDiskStateView that is ready to use. Library modules will be preloaded into the
    /// storage if `load_libraries` is true.
    ///
    /// NOTE: this is the only way to get a state view in Move CLI, and thus, this function needs
    /// to be run before every command that needs a state view, i.e., `check`, `publish`, `run`,
    /// `view`, and `doctor`.
    pub fn prepare_state(&self, load_libraries: bool) -> Result<OnDiskStateView> {
        let state = OnDiskStateView::create(&self.build_dir, &self.storage_dir)?;

        if load_libraries {
            self.mode.prepare(&self.get_package_dir(), false)?;

            // preload the storage with library modules (if such modules do not exist yet)
            let lib_modules = self.get_library_modules()?;
            let new_modules: Vec<_> = lib_modules
                .into_iter()
                .filter(|m| !state.has_module(&m.self_id()))
                .collect();

            let mut serialized_modules = vec![];
            for module in new_modules {
                let mut module_bytes = vec![];
                module.serialize(&mut module_bytes)?;
                serialized_modules.push((module.self_id(), module_bytes));
            }
            state.save_modules(&serialized_modules)?;
        }

        Ok(state)
    }
}

/// Compile the user modules in `src` and the script in `script_file`
fn check(state: OnDiskStateView, republish: bool, files: &[String], verbose: bool) -> Result<()> {
    if verbose {
        println!("Checking Move files...");
    }
    Compiler::new(files, &[state.interface_files_dir()?])
        .set_flags(Flags::empty().set_sources_shadow_deps(republish))
        .check_and_report()?;
    Ok(())
}

fn publish(
    state: OnDiskStateView,
    files: &[String],
    republish: bool,
    ignore_breaking_changes: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Compiling Move modules...")
    }

    let (_, compiled_units) = Compiler::new(files, &[state.interface_files_dir()?])
        .set_flags(Flags::empty().set_sources_shadow_deps(republish))
        .build_and_report()?;

    let num_modules = compiled_units
        .iter()
        .filter(|u| matches!(u, CompiledUnit::Module { .. }))
        .count();
    if verbose {
        println!("Found and compiled {} modules", num_modules)
    }

    let mut modules = vec![];
    for c in compiled_units {
        match c {
            CompiledUnit::Script { loc, .. } => {
                if verbose {
                    println!(
                        "Warning: Found script in specified files for publishing. But scripts \
                         cannot be published. Script found in: {}",
                        loc.file()
                    )
                }
            }
            CompiledUnit::Module { module, .. } => modules.push(module),
        }
    }

    // use the the publish_module API frm the VM if we do not allow breaking changes
    if !ignore_breaking_changes {
        let vm = MoveVM::new(starcoin_natives()).map_err(|m| m.into_vm_status())?;
        let mut cost_strategy = get_cost_strategy(None)?;
        let mut session: SessionAdapter<_> = vm.new_session(&state).into();

        let mut has_error = false;
        for module in &modules {
            let mut module_bytes = vec![];
            module.serialize(&mut module_bytes)?;

            let id = module.self_id();
            let sender = *id.address();

            // check compatibility.
            if state.has_module(&id) {
                let old_module = state.get_compiled_module(&id)?;
                if !Compatibility::check(&Module::new(&old_module), &Module::new(module))
                    .is_fully_compatible()
                {
                    let err = PartialVMError::new(StatusCode::BACKWARD_INCOMPATIBLE_MODULE_UPDATE)
                        .finish(Location::Module(id));
                    explain_publish_error(err, &state, module)?;
                    has_error = true;
                    break;
                }
            }

            let res = session
                .as_mut()
                .publish_module(module_bytes, sender, &mut cost_strategy);
            if let Err(err) = res {
                explain_publish_error(err, &state, module)?;
                has_error = true;
                break;
            }
        }

        if !has_error {
            let (changeset, events) = Into::<Session<_>>::into(session)
                .finish()
                .map_err(|e| e.into_vm_status())?;
            assert!(events.is_empty());
            if verbose {
                explain_publish_changeset(&changeset, &state);
            }
            let modules: Vec<_> = changeset
                .into_modules()
                .map(|(module_id, blob_opt)| (module_id, blob_opt.expect("must be non-deletion")))
                .collect();
            state.save_modules(&modules)?;
        }
    } else {
        // NOTE: the VM enforces the most strict way of module republishing and does not allow
        // backward incompatible changes, as as result, if this flag is set, we skip the VM process
        // and force the CLI to override the on-disk state directly
        let mut serialized_modules = vec![];
        for module in modules {
            let mut module_bytes = vec![];
            module.serialize(&mut module_bytes)?;
            serialized_modules.push((module.self_id(), module_bytes));
        }
        state.save_modules(&serialized_modules)?;
    }

    Ok(())
}

fn run(
    state: OnDiskStateView,
    script_file: &str,
    script_name_opt: &Option<String>,
    signers: &[String],
    txn_args: &[TransactionArgument],
    vm_type_args: Vec<TypeTag>,
    gas_budget: Option<u64>,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let path = Path::new(script_file);
    if !path.exists() {
        bail!("Script file {:?} does not exist", path)
    };
    let bytecode = if Move::is_bytecode_file(path) {
        assert!(
            state.is_module_path(path) || !Move::contains_module(path),
            "Attempting to run module {:?} outside of the `storage/` directory.
move run` must be applied to a module inside `storage/`",
            path
        );
        // script bytecode; read directly from file
        fs::read(path)?
    } else {
        // script source file; compile first and then extract bytecode
        let script_opt = compile_script(&state, script_file, verbose)?;
        match script_opt {
            Some(script) => {
                let mut script_bytes = vec![];
                script.serialize(&mut script_bytes)?;
                script_bytes
            }
            None => bail!("Unable to find script in file {:?}", script_file),
        }
    };

    let signer_addresses = signers
        .iter()
        .map(|s| AccountAddress::from_hex_literal(s))
        .collect::<Result<Vec<AccountAddress>, _>>()?;
    // TODO: parse Value's directly instead of going through the indirection of TransactionArgument?
    let vm_args: Vec<Vec<u8>> = convert_txn_args(txn_args);

    let vm = MoveVM::new(starcoin_natives()).map_err(|m| m.into_vm_status())?;
    let mut cost_strategy = get_cost_strategy(gas_budget)?;
    let mut session: SessionAdapter<_> = vm.new_session(&state).into();

    let script_type_parameters = vec![];
    let script_parameters = vec![];
    let res = match script_name_opt {
        Some(script_name) => {
            // script fun. parse module, extract script ID to pass to VM
            let module = CompiledModule::deserialize(&bytecode)
                .map_err(|e| anyhow!("Error deserializing module: {:?}", e))?;
            session
                .as_mut()
                .execute_script_function(
                    &module.self_id(),
                    IdentStr::new(script_name)?,
                    vm_type_args.clone(),
                    vm_args,
                    signer_addresses.clone(),
                    &mut cost_strategy,
                )
                .map(|_| ())
        }
        None => session.as_mut().execute_script(
            bytecode.to_vec(),
            vm_type_args.clone(),
            vm_args,
            signer_addresses.clone(),
            &mut cost_strategy,
        ),
    };

    if let Err(err) = res {
        explain_execution_error(
            err,
            &state,
            &script_type_parameters,
            &script_parameters,
            &vm_type_args,
            &signer_addresses,
            txn_args,
        )
    } else {
        let (changeset, events) = Into::<Session<_>>::into(session)
            .finish()
            .map_err(|e| e.into_vm_status())?;
        if verbose {
            explain_execution_effects(&changeset, &events, &state)?
        }
        maybe_commit_effects(!dry_run, changeset, events, &state)
    }
}

fn run_on_remote<R: MoveStorage>(
    state: OnDiskStateView,
    remote_state: R,
    script_file: &str,
    script_name_opt: &Option<String>,
    signers: &[String],
    txn_args: &[TransactionArgument],
    vm_type_args: Vec<TypeTag>,
    gas_budget: Option<u64>,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let path = Path::new(script_file);
    if !path.exists() {
        bail!("Script file {:?} does not exist", path)
    };
    let bytecode = if Move::is_bytecode_file(path) {
        assert!(
            state.is_module_path(path) || !Move::contains_module(path),
            "Attempting to run module {:?} outside of the `storage/` directory.
move run` must be applied to a module inside `storage/`",
            path
        );
        // script bytecode; read directly from file
        fs::read(path)?
    } else {
        // script source file; compile first and then extract bytecode
        let script_opt = compile_script(&state, script_file, verbose)?;
        match script_opt {
            Some(script) => {
                let mut script_bytes = vec![];
                script.serialize(&mut script_bytes)?;
                script_bytes
            }
            None => bail!("Unable to find script in file {:?}", script_file),
        }
    };

    let signer_addresses = signers
        .iter()
        .map(|s| AccountAddress::from_hex_literal(s))
        .collect::<Result<Vec<AccountAddress>, _>>()?;
    // TODO: parse Value's directly instead of going through the indirection of TransactionArgument?
    let vm_args: Vec<Vec<u8>> = convert_txn_args(txn_args);

    let cost_strategy = get_cost_strategy(gas_budget)?;
    let script_type_parameters = vec![];
    let script_parameters = vec![];
    let merged_state = MergedRemoteCache {
        a: state,
        b: remote_state,
    };
    let res = execute(
        &merged_state,
        script_name_opt,
        bytecode,
        signer_addresses.clone(),
        vm_args,
        vm_type_args.clone(),
        cost_strategy,
    );

    match res {
        Err(err) => explain_execution_error(
            err,
            &merged_state,
            &script_type_parameters,
            &script_parameters,
            &vm_type_args,
            &signer_addresses,
            txn_args,
        ),
        Ok((changeset, events)) => {
            if verbose {
                explain_execution_effects(&changeset, &events, &merged_state)?;
            }
            maybe_commit_effects(!dry_run, changeset, events, &merged_state.a)
        }
    }
}

fn compile_script(
    state: &OnDiskStateView,
    script_file: &str,
    verbose: bool,
) -> Result<Option<CompiledScript>> {
    if verbose {
        println!("Compiling transaction script...")
    }
    let (_, compiled_units) =
        Compiler::new(&[script_file.to_string()], &[state.interface_files_dir()?])
            .set_flags(Flags::empty().set_sources_shadow_deps(true))
            .build_and_report()?;

    let mut script_opt = None;
    for c in compiled_units {
        match c {
            CompiledUnit::Script { script, .. } => {
                if script_opt.is_some() {
                    bail!("Error: Found more than one script")
                }
                script_opt = Some(script)
            }
            CompiledUnit::Module { ident, .. } => {
                if verbose {
                    println!(
                        "Warning: Found module '{}' in file specified for the script. This \
                             module will not be published.",
                        ident.module_name
                    )
                }
            }
        }
    }

    Ok(script_opt)
}

// execute the script against the given state
fn execute<R: MoveStorage>(
    state: &R,
    script_name_opt: &Option<String>,
    bytecode: Vec<u8>,
    signer_addresses: Vec<AccountAddress>,
    vm_args: Vec<Vec<u8>>,
    vm_type_args: Vec<TypeTag>,
    mut cost_strategy: GasStatus,
) -> Result<(ChangeSet, Vec<Event>), VMError> {
    let vm = MoveVM::new(starcoin_natives())?;
    let mut session: SessionAdapter<_> = vm.new_session(state).into();
    let res = match script_name_opt {
        Some(script_name) => {
            // script fun. parse module, extract script ID to pass to VM
            let module = CompiledModule::deserialize(&bytecode)
                .map_err(|e| e.finish(Location::Undefined))?;

            session
                .as_mut()
                .execute_script_function(
                    &module.self_id(),
                    IdentStr::new(script_name).map_err(|_| {
                        PartialVMError::new(StatusCode::UNEXPECTED_DESERIALIZATION_ERROR)
                            .finish(Location::Undefined)
                    })?,
                    vm_type_args,
                    vm_args,
                    signer_addresses,
                    &mut cost_strategy,
                )
                .map(|_| ())
        }
        None => session.as_mut().execute_script(
            bytecode.to_vec(),
            vm_type_args,
            vm_args,
            signer_addresses,
            &mut cost_strategy,
        ),
    };

    res.and_then(|_| Into::<Session<_>>::into(session).finish())
}

fn get_cost_strategy(gas_budget: Option<u64>) -> Result<GasStatus<'static>> {
    let gas_schedule = &INITIAL_GAS_SCHEDULE;
    let cost_strategy = if let Some(gas_budget) = gas_budget {
        let max_gas_budget = u64::MAX
            .checked_div(gas_schedule.gas_constants.gas_unit_scaling_factor)
            .unwrap();
        if gas_budget >= max_gas_budget {
            bail!("Gas budget set too high; maximum is {}", max_gas_budget)
        }
        GasStatus::new(gas_schedule, GasUnits::new(gas_budget))
    } else {
        // no budget specified. use CostStrategy::system, which disables gas metering
        let mut gas_status = GasStatus::new(gas_schedule, GasUnits::new(0));
        gas_status.set_metering(false);
        gas_status
    };
    Ok(cost_strategy)
}

fn explain_publish_changeset(changeset: &ChangeSet, state: &OnDiskStateView) {
    // publish effects should contain no resources
    assert!(changeset.resources().next().is_none());
    // total bytes written across all accounts
    let mut total_bytes_written = 0;
    for (addr, name, blob_opt) in changeset.modules() {
        if let Some(module_bytes) = blob_opt {
            let bytes_written = addr.len() + name.len() + module_bytes.len();
            total_bytes_written += bytes_written;
            let module_id = ModuleId::new(addr, name.clone());
            if state.has_module(&module_id) {
                println!(
                    "Updating an existing module {} (wrote {:?} bytes)",
                    module_id, bytes_written
                );
            } else {
                println!(
                    "Publishing a new module {} (wrote {:?} bytes)",
                    module_id, bytes_written
                );
            }
        } else {
            panic!("Deleting a module is not supported")
        }
    }
    println!(
        "Wrote {:?} bytes of module ID's and code",
        total_bytes_written
    )
}

fn explain_execution_effects<R: MoveStorage>(
    changeset: &ChangeSet,
    events: &[Event],
    state: &R,
) -> Result<()> {
    // execution effects should contain no modules
    assert!(changeset.modules().next().is_none());
    if !events.is_empty() {
        println!("Emitted {:?} events:", events.len());
        // TODO: better event printing
        for (event_key, event_sequence_number, _event_type, event_data) in events {
            println!(
                "Emitted {:?} as the {}th event to stream {:?}",
                event_data, event_sequence_number, event_key
            )
        }
    }
    if !changeset.accounts().is_empty() {
        println!(
            "Changed resource(s) under {:?} address(es):",
            changeset.accounts().len()
        );
    }
    // total bytes written across all accounts
    let mut total_bytes_written = 0;
    for (addr, account) in changeset.accounts() {
        print!("  ");
        if account.resources().is_empty() {
            continue;
        }
        println!(
            "Changed {:?} resource(s) under address {:?}:",
            account.resources().len(),
            addr
        );
        for (struct_tag, write_opt) in account.resources() {
            print!("    ");
            let mut bytes_to_write = struct_tag.access_vector().len();
            match write_opt {
                Some(blob) => {
                    bytes_to_write += blob.len();
                    if state
                        .get_resource(addr, struct_tag)
                        .map_err(|e| e.finish(Location::Undefined).into_vm_status())?
                        .is_some()
                    {
                        // TODO: print resource diff
                        println!(
                            "Changed type {}: {:?} (wrote {:?} bytes)",
                            struct_tag, blob, bytes_to_write
                        )
                    } else {
                        // TODO: nicer printing
                        println!(
                            "Added type {}: {:?} (wrote {:?} bytes)",
                            struct_tag, blob, bytes_to_write
                        )
                    }
                }
                None => println!(
                    "Deleted type {} (wrote {:?} bytes)",
                    struct_tag, bytes_to_write
                ),
            };
            total_bytes_written += bytes_to_write;
        }
    }
    if total_bytes_written != 0 {
        println!(
            "Wrote {:?} bytes of resource ID's and data",
            total_bytes_written
        );
    }

    Ok(())
}

/// Commit the resources and events modified by a transaction to disk
fn maybe_commit_effects(
    commit: bool,
    changeset: ChangeSet,
    events: Vec<Event>,
    state: &OnDiskStateView,
) -> Result<()> {
    // similar to explain effects, all module publishing happens via save_modules(), so effects
    // shouldn't contain modules
    if commit {
        for (addr, account) in changeset.into_inner() {
            let (_modules, resources) = account.into_inner();
            for (struct_tag, blob_opt) in resources {
                match blob_opt {
                    Some(blob) => state.save_resource(addr, struct_tag, &blob)?,
                    None => state.delete_resource(addr, struct_tag)?,
                }
            }
        }

        for (event_key, event_sequence_number, event_type, event_data) in events {
            state.save_event(&event_key, event_sequence_number, event_type, event_data)?
        }
    } else if !(changeset.resources().next().is_none() && events.is_empty()) {
        println!("Discarding changes; re-run without --dry-run if you would like to keep them.")
    }

    Ok(())
}

fn explain_type_error(
    script_params: &[SignatureToken],
    signers: &[AccountAddress],
    txn_args: &[TransactionArgument],
) {
    use SignatureToken::*;
    let expected_num_signers = script_params
        .iter()
        .filter(|t| match t {
            Reference(r) => r.is_signer(),
            _ => false,
        })
        .count();
    if expected_num_signers != signers.len() {
        println!(
            "Execution failed with incorrect number of signers: script expected {:?}, but found \
             {:?}",
            expected_num_signers,
            signers.len()
        );
        return;
    }

    // TODO: printing type(s) of missing arguments could be useful
    let expected_num_args = script_params.len() - signers.len();
    if expected_num_args != txn_args.len() {
        println!(
            "Execution failed with incorrect number of arguments: script expected {:?}, but found \
             {:?}",
            expected_num_args,
            txn_args.len()
        );
        return;
    }

    // TODO: print more helpful error message pinpointing the (argument, type)
    // pair that didn't match
    println!("Execution failed with type error when binding type arguments to type parameters")
}

fn explain_publish_error(
    error: VMError,
    state: &OnDiskStateView,
    module: &CompiledModule,
) -> Result<()> {
    use StatusCode::*;

    let module_id = module.self_id();
    match error.into_vm_status() {
        VMStatus::Error(DUPLICATE_MODULE_NAME) => {
            println!(
                "Module {} exists already. Re-run without --no-republish to publish anyway.",
                module_id
            );
        }
        VMStatus::Error(BACKWARD_INCOMPATIBLE_MODULE_UPDATE) => {
            println!("Breaking change detected--publishing aborted. Re-run with --ignore-breaking-changes to publish anyway.");

            let old_module = state.get_compiled_module(&module_id)?;
            let old_api = normalized::Module::new(&old_module);
            let new_api = normalized::Module::new(module);
            let compat = Compatibility::check(&old_api, &new_api);
            // the only way we get this error code is compatibility checking failed, so assert here
            assert!(!compat.is_fully_compatible());

            if !compat.struct_layout {
                // TODO: we could choose to make this more precise by walking the global state and looking for published
                // structs of this type. but probably a bad idea
                println!("Layout API for structs of module {} has changed. Need to do a data migration of published structs", module_id)
            } else if !compat.struct_and_function_linking {
                // TODO: this will report false positives if we *are* simultaneously redeploying all dependent modules.
                // but this is not easy to check without walking the global state and looking for everything
                println!("Linking API for structs/functions of module {} has changed. Need to redeploy all dependent modules.", module_id)
            }
        }
        VMStatus::Error(CYCLIC_MODULE_DEPENDENCY) => {
            println!(
                "Publishing module {} introduces cyclic dependencies.",
                module_id
            );
            // find all cycles with an iterative DFS
            let code_cache = state.get_code_cache()?;

            let mut stack = vec![];
            let mut state = BTreeMap::new();
            state.insert(module_id.clone(), true);
            for dep in module.immediate_dependencies() {
                stack.push((code_cache.get_module(&dep)?, false));
            }

            while !stack.is_empty() {
                let (cur, is_exit) = stack.pop().unwrap();
                let cur_id = cur.self_id();
                if is_exit {
                    state.insert(cur_id, false);
                } else {
                    state.insert(cur_id, true);
                    stack.push((cur, true));
                    for next in cur.immediate_dependencies() {
                        if let Some(is_discovered_but_not_finished) = state.get(&next) {
                            if *is_discovered_but_not_finished {
                                let cycle_path: Vec<_> = stack
                                    .iter()
                                    .filter(|(_, is_exit)| *is_exit)
                                    .map(|(m, _)| m.self_id().to_string())
                                    .collect();
                                println!(
                                    "Cycle detected: {} -> {} -> {}",
                                    module_id,
                                    cycle_path.join(" -> "),
                                    module_id,
                                );
                            }
                        } else {
                            stack.push((code_cache.get_module(&next)?, false));
                        }
                    }
                }
            }
            println!("Re-run with --ignore-breaking-changes to publish anyway.")
        }
        VMStatus::Error(status_code) => {
            println!("Publishing failed with unexpected error {:?}", status_code)
        }
        VMStatus::Executed | VMStatus::MoveAbort(..) | VMStatus::ExecutionFailure { .. } => {
            unreachable!()
        }
    }

    Ok(())
}

/// Explain an execution error
fn explain_execution_error<R: MoveStorage>(
    error: VMError,
    state: &R,
    script_type_parameters: &[AbilitySet],
    script_parameters: &[SignatureToken],
    vm_type_args: &[TypeTag],
    signers: &[AccountAddress],
    txn_args: &[TransactionArgument],
) -> Result<()> {
    use StatusCode::*;
    match error.into_vm_status() {
        VMStatus::MoveAbort(AbortLocation::Module(id), abort_code) => {
            // try to use move-explain to explain the abort
            // TODO: this will only work for errors in the stdlib or Diem Framework. We should
            // add code to build an ErrorMapping for modules in move_lib as well
            let error_descriptions: ErrorMapping = bcs::from_bytes(stdlib::ERROR_DESCRIPTIONS)?;
            print!(
                "Execution aborted with code {} in module {}.",
                abort_code, id
            );

            if let Some(error_desc) = error_descriptions.get_explanation(&id, abort_code) {
                println!(
                    " Abort code details:\nReason:\n  Name: {}\n  Description:{}\nCategory:\n  \
                     Name: {}\n  Description:{}",
                    error_desc.reason.code_name,
                    error_desc.reason.code_description,
                    error_desc.category.code_name,
                    error_desc.category.code_description,
                )
            } else {
                println!()
            }
        }
        VMStatus::MoveAbort(AbortLocation::Script, abort_code) => {
            // TODO: map to source code location
            println!(
                "Execution aborted with code {} in transaction script",
                abort_code
            )
        }
        VMStatus::ExecutionFailure {
            status_code,
            location,
            function,
            code_offset,
        } => {
            let status_explanation = match status_code {
                RESOURCE_ALREADY_EXISTS => "a RESOURCE_ALREADY_EXISTS error (i.e., \
                                            `move_to<T>(account)` when there is already a \
                                            resource of type `T` under `account`)"
                    .to_string(),
                MISSING_DATA => "a RESOURCE_DOES_NOT_EXIST error (i.e., `move_from<T>(a)`, \
                                 `borrow_global<T>(a)`, or `borrow_global_mut<T>(a)` when there \
                                 is no resource of type `T` at address `a`)"
                    .to_string(),
                ARITHMETIC_ERROR => "an arithmetic error (i.e., integer overflow/underflow, \
                                     div/mod by zero, or invalid shift)"
                    .to_string(),
                EXECUTION_STACK_OVERFLOW => "an execution stack overflow".to_string(),
                CALL_STACK_OVERFLOW => "a call stack overflow".to_string(),
                OUT_OF_GAS => "an out of gas error".to_string(),
                _ => format!("a {} error", status_code.status_type()),
            };
            // TODO: map to source code location
            let location_explanation = match location {
                AbortLocation::Module(id) => {
                    format!("{}::{}", id, state.resolve_function(&id, function)?)
                }
                AbortLocation::Script => "script".to_string(),
            };
            println!(
                "Execution failed because of {} in {} at code offset {}",
                status_explanation, location_explanation, code_offset
            )
        }
        VMStatus::Error(NUMBER_OF_TYPE_ARGUMENTS_MISMATCH) => println!(
            "Execution failed with incorrect number of type arguments: script expected {:?}, but \
             found {:?}",
            script_type_parameters.len(),
            vm_type_args.len()
        ),
        VMStatus::Error(TYPE_MISMATCH) => explain_type_error(script_parameters, signers, txn_args),
        VMStatus::Error(LINKER_ERROR) => {
            // TODO: is this the only reason we can see LINKER_ERROR?
            // Can we also see it if someone manually deletes modules in storage?
            println!(
                "Execution failed due to unresolved type argument(s) (i.e., `--type-args \
                 0x1::M:T` when there is no module named M at 0x1 or no type named T in module \
                 0x1::M)"
            );
        }
        VMStatus::Error(status_code) => {
            println!("Execution failed with unexpected error {:?}", status_code)
        }
        VMStatus::Executed => unreachable!(),
    }
    Ok(())
}

/// Print a module or resource stored in `file`
fn view(state: OnDiskStateView, file: &str) -> Result<()> {
    let path = Path::new(&file);
    if state.is_resource_path(path) {
        match state.view_resource(path)? {
            Some(resource) => println!("{}", resource),
            None => println!("Resource not found."),
        }
    } else if state.is_event_path(path) {
        let events = state.view_events(path)?;
        if events.is_empty() {
            println!("Events not found.")
        } else {
            for event in events {
                println!("{}", event)
            }
        }
    } else if Move::is_bytecode_file(path) {
        let bytecode_opt = if Move::contains_module(path) {
            OnDiskStateView::view_module(path)?
        } else {
            // bytecode extension, but not a module--assume it's a script
            OnDiskStateView::view_script(path)?
        };
        match bytecode_opt {
            Some(bytecode) => println!("{}", bytecode),
            None => println!("Bytecode not found."),
        }
    } else {
        bail!("`move view <file>` must point to a valid file under storage")
    }
    Ok(())
}

/// Run sanity checks on storage and build dirs. This is primarily intended for testing the CLI;
/// doctor should never fail unless `publish --ignore-breaking changes` is used or files under
/// `storage` or `build` are modified manually. This runs the following checks:
/// (1) all modules pass the bytecode verifier
/// (2) all modules pass the linker
/// (3) all resources can be deserialized
/// (4) all events can be deserialized
/// (5) build/mv_interfaces is consistent with the global storage (TODO?)
fn doctor(state: OnDiskStateView) -> Result<()> {
    fn parent_addr(p: &Path) -> &OsStr {
        p.parent().unwrap().parent().unwrap().file_name().unwrap()
    }

    // verify and link each module
    let code_cache = state.get_code_cache()?;
    for module in code_cache.all_modules() {
        if bytecode_verifier::verify_module(module).is_err() {
            bail!("Failed to verify module {:?}", module.self_id())
        }

        let imm_deps = code_cache.get_immediate_module_dependencies(module)?;
        if bytecode_verifier::dependencies::verify_module(module, imm_deps).is_err() {
            bail!(
                "Failed to link module {:?} against its dependencies",
                module.self_id()
            )
        }

        let cyclic_check_result = bytecode_verifier::cyclic_dependencies::verify_module(
            module,
            |module_id| {
                code_cache
                    .get_module(module_id)
                    .map_err(|_| PartialVMError::new(StatusCode::MISSING_DEPENDENCY))
                    .map(|m| m.immediate_dependencies())
            },
            |module_id| {
                code_cache
                    .get_module(module_id)
                    .map_err(|_| PartialVMError::new(StatusCode::MISSING_DEPENDENCY))
                    .map(|m| m.immediate_friends())
            },
        );
        if let Err(cyclic_check_error) = cyclic_check_result {
            // the only possible error in the CLI's context is CYCLIC_MODULE_DEPENDENCY
            assert_eq!(
                cyclic_check_error.major_status(),
                StatusCode::CYCLIC_MODULE_DEPENDENCY
            );
            bail!(
                "Cyclic module dependencies are detected with module {} in the loop",
                module.self_id()
            )
        }
    }
    // deserialize each resource
    for resource_path in state.resource_paths() {
        let resource = state.view_resource(&resource_path);
        if resource.is_err() {
            bail!(
                "Failed to deserialize resource {:?} stored under address {:?}",
                resource_path.file_name().unwrap(),
                parent_addr(&resource_path)
            )
        }
    }
    // deserialize each event
    for event_path in state.event_paths() {
        let event = state.view_events(&event_path);
        if event.is_err() {
            bail!(
                "Failed to deserialize event {:?} stored under address {:?}",
                event_path.file_name().unwrap(),
                parent_addr(&event_path)
            )
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let move_args: Move = Move::from_args();
    match &move_args.cmd {
        Command::Scaffold { path } => test::create_test_scaffold(path),
        Command::Check {
            source_files,
            no_republish,
        } if move_args.mode.1 == DepMode::OnChain => {
            let state = move_args.prepare_state(true)?;
            // get deps first.
            let view =
                RemoteStateView::from_url(move_args.starcoin_rpc.as_str(), move_args.block_number)?;

            // stdlib modules always included.
            let mut all_module_deps = view
                .get_modules(core_code_address())
                .map_err(|e| e.into_vm_status())?
                .expect("stdlib exists on chain")
                .into_iter()
                .map(|(k, v)| (ModuleId::new(core_code_address(), k), v))
                .collect::<BTreeMap<_, _>>();

            let view = MergedRemoteCache { a: state, b: view };
            let mut found_modules = resolve_deps(&view, source_files)?;
            let module_deps = view.get_module_dependencies_recursively_for_all(&found_modules)?;
            found_modules.extend(module_deps.values().cloned());

            for x in found_modules.iter().chain(module_deps.values()) {
                all_module_deps.entry(x.self_id()).or_insert_with(|| {
                    let mut blob = vec![];
                    x.serialize(&mut blob).unwrap();
                    blob
                });
            }

            view.a
                .save_modules(all_module_deps.into_iter().collect::<Vec<_>>().iter())?;

            check(view.a, !*no_republish, source_files, move_args.verbose)
        }
        Command::Check {
            source_files,
            no_republish,
        } => {
            let state = move_args.prepare_state(true)?;
            check(state, !*no_republish, source_files, move_args.verbose)
        }
        Command::Publish {
            source_files,
            no_republish,
            ignore_breaking_changes,
        } => {
            let state = move_args.prepare_state(false)?;
            publish(
                state,
                source_files,
                !*no_republish,
                *ignore_breaking_changes,
                move_args.verbose,
            )
        }
        Command::Run {
            script_file,
            script_name,
            signers,
            args,
            type_args,
            gas_budget,
            dry_run,
        } if move_args.mode.1 == DepMode::OnChain => {
            let local_state = move_args.prepare_state(false)?;
            let remote_state =
                RemoteStateView::from_url(move_args.starcoin_rpc.as_str(), move_args.block_number)?;
            run_on_remote(
                local_state,
                remote_state,
                script_file,
                script_name,
                signers,
                args,
                type_args.to_vec(),
                *gas_budget,
                *dry_run,
                move_args.verbose,
            )
        }
        Command::Run {
            script_file,
            script_name,
            signers,
            args,
            type_args,
            gas_budget,
            dry_run,
        } => {
            let state = move_args.prepare_state(false)?;
            run(
                state,
                script_file,
                script_name,
                signers,
                args,
                type_args.to_vec(),
                *gas_budget,
                *dry_run,
                move_args.verbose,
            )
        }
        Command::FunctionalTest {
            filter,
            filter_exact,
            test_threads,
            quiet,
            list,
            ignored,
        } => {
            let opts = datatest_stable::TestOpts {
                filter: filter.clone(),
                filter_exact: *filter_exact,
                test_threads: *test_threads,
                quiet: *quiet,
                nocapture: false,
                list: *list,
                ignored: *ignored,
                include_ignored: false,
                force_run_in_process: false,
                exclude_should_panic: false,
                bench: false,
                test: false,
                logfile: None,
                skip: vec![],
                show_output: false,
                color: None,
                format: Default::default(),
                report_time: None,
                ensure_time: false,
            };
            // let inteface_dir = state.interface_files_dir()?;
            let mut requirements = Vec::new();

            let filter = filter.clone().unwrap_or_else(|| r".*\.move".to_string());
            requirements.push(datatest_stable::Requirements::new(
                |path| {
                    std::env::set_var(PRETTY, "true");
                    // let mut deps = restore_stdlib_in_dir(temp_dir.path())?;
                    let mut deps = vec![];
                    {
                        let build_dir = DEFAULT_BUILD_DIR.to_string();
                        let mv_interface_dir = PathBuf::from(build_dir)
                            .join(MOVE_COMPILED_INTERFACES_DIR)
                            .display()
                            .to_string();
                        deps.push(mv_interface_dir);
                    }

                    let compiler = crate::functional_test::MoveSourceCompiler::new(deps);

                    let mut exec = FakeExecutor::new();

                    // add source modules to state.
                    {
                        let storage_dir = DEFAULT_STORAGE_DIR;
                        for compiled_module_path in walkdir::WalkDir::new(storage_dir)
                            .into_iter()
                            .filter_map(|e| e.ok().filter(|d| d.path().is_file()))
                            .map(|e| e.path().to_path_buf())
                            .filter(move |path| {
                                path.parent()
                                    .map(|p| p.ends_with("modules"))
                                    .unwrap_or_default()
                            })
                        {
                            let compiled_module =
                                CompiledModule::deserialize(&fs::read(compiled_module_path)?)?;
                            exec.add_module(&compiled_module.self_id(), &compiled_module);
                        }
                    }

                    starcoin_functional_tests::testsuite::functional_tests_with_executor(
                        compiler, &mut exec, path,
                    )
                },
                "functional-test".to_string(),
                DEFAULT_TEST_DIR.to_string(),
                filter,
            ));
            datatest_stable::runner_with_opts(&requirements, opts);
            Ok(())
        }
        Command::UnitTest {
            instruction_execution_bound,
            filter,
            list,
            num_threads,
            report_statistics,
            report_storage_on_error,
            source_files,
            check_stackless_vm,
        } => {
            let mut sources = source_files.clone();
            // only support packages deps
            sources.push(move_args.get_package_dir().display().to_string());

            let testing_config = UnitTestingConfig {
                instruction_execution_bound: *instruction_execution_bound,
                filter: filter.clone(),
                list: *list,
                num_threads: *num_threads,
                dep_files: vec![],
                report_statistics: *report_statistics,
                report_storage_on_error: *report_storage_on_error,
                source_files: sources,
                check_stackless_vm: *check_stackless_vm,
                verbose: move_args.verbose,
            };
            let test_plan = testing_config.build_test_plan();
            if let Some(test_plan) = test_plan {
                testing_config.run_and_report_unit_tests(
                    test_plan,
                    Some(starcoin_natives()),
                    std::io::stdout(),
                )?;
            }
            Ok(())
        }
        Command::Test {
            path,
            track_cov: _,
            create: true,
        } => test::create_test_scaffold(path),
        Command::Test {
            path,
            track_cov,
            create: false,
        } => test::run_all(
            path,
            &std::env::current_exe()?.to_string_lossy(),
            *track_cov,
        ),
        Command::View { file } => {
            let state = move_args.prepare_state(false)?;
            view(state, file)
        }
        Command::Clean {} => {
            // delete storage
            let storage_dir = Path::new(&move_args.storage_dir);
            if storage_dir.exists() {
                fs::remove_dir_all(&storage_dir)?;
            }

            // delete build
            let build_dir = Path::new(&move_args.build_dir);
            if build_dir.exists() {
                fs::remove_dir_all(&build_dir)?;
            }
            Ok(())
        }
        Command::Doctor {} => {
            let state = move_args.prepare_state(false)?;
            doctor(state)
        }
    }
}
