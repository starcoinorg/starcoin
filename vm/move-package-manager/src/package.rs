use clap::Parser;
use move_cli::base::coverage::CoverageSummaryOptions;
use move_cli::base::prove::ProverOptions;
use move_vm_runtime::native_functions::NativeFunctionTable;
use std::path::PathBuf;

#[derive(Parser)]
pub enum PackageCommand {
    /// Create a new Move package with name `name` at `path`. If `path` is not provided the package
    /// will be created in the directory `name`.
    #[clap(name = "new")]
    New {
        /// The name of the package to be created.
        name: String,
    },
    /// Build the package at `path`. If no path is provided defaults to current directory.
    #[clap(name = "build")]
    Build,
    /// Print address information.
    #[clap(name = "info")]
    Info,
    /// Generate error map for the package and its dependencies at `path` for use by the Move
    /// explanation tool.
    #[clap(name = "errmap")]
    ErrMapGen {
        /// The prefix that all error reasons within modules will be prefixed with, e.g., "E" if
        /// all error reasons are "E_CANNOT_PERFORM_OPERATION", "E_CANNOT_ACCESS", etc.
        #[clap(long)]
        error_prefix: Option<String>,
        /// The file to serialize the generated error map to.
        #[clap(long, default_value = "error_map", parse(from_os_str))]
        output_file: PathBuf,
    },
    /// Run the Move Prover on the package at `path`. If no path is provided defaults to current
    /// directory. Use `.. prove .. -- <options>` to pass on options to the prover.
    #[clap(name = "prove")]
    Prove {
        /// The target filter used to prune the modules to verify. Modules with a name that contains
        /// this string will be part of verification.
        #[clap(short = 't', long = "target")]
        target_filter: Option<String>,
        /// Internal field indicating that this prover run is for a test.
        #[clap(skip)]
        for_test: bool,
        /// Any options passed to the prover.
        #[clap(subcommand)]
        options: Option<ProverOptions>,
    },
    /// Inspect test coverage for this package. A previous test run with the `--coverage` flag must
    /// have previously been run.
    #[clap(name = "coverage")]
    CoverageReport {
        #[clap(subcommand)]
        options: CoverageSummaryOptions,
    },
    /// Run Move unit tests in this package.
    #[clap(name = "test")]
    UnitTest {
        /// Bound the number of instructions that can be executed by any one test.
        #[clap(
            name = "instructions",
            default_value = "5000",
            short = 'i',
            long = "instructions"
        )]
        instruction_execution_bound: u64,
        /// A filter string to determine which unit tests to run. A unit test will be run only if it
        /// contains this string in its fully qualified (<addr>::<module_name>::<fn_name>) name.
        #[clap(name = "filter", short = 'f', long = "filter")]
        filter: Option<String>,
        /// List all tests
        #[clap(name = "list", short = 'l', long = "list")]
        list: bool,
        /// Number of threads to use for running tests.
        #[clap(
            name = "num_threads",
            default_value = "8",
            short = 't',
            long = "threads"
        )]
        num_threads: usize,
        /// Report test statistics at the end of testing
        #[clap(name = "report_statistics", short = 's', long = "statistics")]
        report_statistics: bool,
        /// Show the storage state at the end of execution of a failing test
        #[clap(name = "global_state_on_error", short = 'g', long = "state_on_error")]
        report_storage_on_error: bool,
        /// Use the stackless bytecode interpreter to run the tests and cross check its results with
        /// the execution result from Move VM.
        #[clap(long = "stackless")]
        check_stackless_vm: bool,
        /// Verbose mode
        #[clap(long = "verbose")]
        verbose_mode: bool,
        /// Collect coverage information for later use with the various `package coverage` subcommands
        #[clap(long = "coverage")]
        compute_coverage: bool,

        /// Use the EVM-based execution backend.
        /// Does not work with --stackless.
        #[cfg(feature = "evm-backend")]
        #[structopt(long = "evm")]
        evm: bool,
    },
    /// Disassemble the Move bytecode pointed to
    #[clap(name = "disassemble")]
    BytecodeView {
        /// Start a disassembled bytecode-to-source explorer
        #[clap(long = "interactive")]
        interactive: bool,
        /// The package name. If not provided defaults to current package modules only
        #[clap(long = "package")]
        package_name: Option<String>,
        /// The name of the module or script in the package to disassemble
        #[clap(long = "name")]
        module_or_script_name: String,
    },
}
pub fn handle_package_commands(
    _path: &Option<PathBuf>,
    _config: move_package::BuildConfig,
    _cmd: &PackageCommand,
    _natives: NativeFunctionTable,
) -> anyhow::Result<()> {
    // This is the exceptional command as it doesn't need a package to run, so we can't count on
    // being able to root ourselves.
    /*
    if let PackageCommand::New { name } = cmd {
        let creation_path = Path::new(&path).join(name);
        create_move_package(name, &creation_path)?;
        return Ok(());
    } */
    Ok(())
}
