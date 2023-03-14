use clap::Parser;
use move_cli::base::{
    build::Build, coverage::Coverage, disassemble::Disassemble, errmap::Errmap, info::Info,
    new::New, prove::Prove, test::Test,
};
use move_cli::Move;
use move_vm_runtime::native_functions::NativeFunctionTable;

pub const STARCOIN_STDLIB_PACKAGE_NAME: &str = "StarcoinFraework";
pub const STARCOIN_STDLIB_PACKAGE_PATH: &str = "{ \
    git = \"https://github.com/starcoinorg/starcoin-framework.git\", rev = \"main\" \
}";
pub const STARCOIN_STDLIB_ADDR_NAME: &str = "StarcoinFraework";
pub const STARCOIN_STDLIB_ADDR_VALUE: &str = "0x1";

#[derive(Parser)]
pub enum PackageCommand {
    /// Create a new Move package with name `name` at `path`. If `path` is not provided the package
    /// will be created in the directory `name`.
    #[clap(name = "new")]
    New(New),
    /// Build the package at `path`. If no path is provided defaults to current directory.
    #[clap(name = "build")]
    Build(Build),
    /// Print address information.
    #[clap(name = "info")]
    Info(Info),
    /// Generate error map for the package and its dependencies at `path` for use by the Move
    /// explanation tool.
    #[clap(name = "errmap")]
    Errmap(Errmap),
    /// Run the Move Prover on the package at `path`. If no path is provided defaults to current
    /// directory. Use `.. prove .. -- <options>` to pass on options to the prover.
    #[clap(name = "prove")]
    Prove(Prove),
    /// Inspect test coverage for this package. A previous test run with the `--coverage` flag must
    /// have previously been run.
    #[clap(name = "coverage")]
    Coverage(Coverage),
    /// Run Move unit tests in this package.
    #[clap(name = "test")]
    Test(Test),
    /// Disassemble the Move bytecode pointed to
    #[clap(name = "disassemble")]
    Disassemble(Disassemble),
}
pub fn handle_package_commands(
    natives: NativeFunctionTable,
    move_args: Move,
    cmd: PackageCommand,
) -> anyhow::Result<()> {
    match cmd {
        PackageCommand::New(c) => c.execute(
            move_args.package_path,
            "0.0.0",
            [(STARCOIN_STDLIB_PACKAGE_NAME, STARCOIN_STDLIB_PACKAGE_PATH)],
            [(STARCOIN_STDLIB_ADDR_NAME, STARCOIN_STDLIB_ADDR_VALUE)],
            "",
        ),
        PackageCommand::Build(c) => c.execute(move_args.package_path, move_args.build_config),
        PackageCommand::Info(c) => c.execute(move_args.package_path, move_args.build_config),
        PackageCommand::Errmap(c) => c.execute(move_args.package_path, move_args.build_config),
        PackageCommand::Prove(c) => c.execute(move_args.package_path, move_args.build_config),
        PackageCommand::Coverage(c) => c.execute(move_args.package_path, move_args.build_config),
        PackageCommand::Test(c) => c.execute(
            move_args.package_path,
            move_args.build_config,
            natives,
            None,
        ),
        PackageCommand::Disassemble(c) => c.execute(move_args.package_path, move_args.build_config),
    }
}
