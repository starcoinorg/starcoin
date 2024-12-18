use clap::Parser;
use move_cli::base::{
    build::Build, coverage::Coverage, disassemble::Disassemble, errmap::Errmap, new::New,
    prove::Prove, test::Test,
};
use move_cli::Move;
use move_vm_runtime::native_functions::NativeFunctionTable;
use starcoin_framework::extended_checks;
use starcoin_vm_runtime::natives;
use starcoin_vm_types::on_chain_config::starcoin_test_feature_flags_genesis;

pub const STARCOIN_STDLIB_PACKAGE_NAME: &str = "starcoin_framework";
pub const STARCOIN_STDLIB_PACKAGE_PATH: &str = "{ \
    git = \"https://github.com/starcoinorg/starcoin-framework.git\", rev = \"main\" \
}";
pub const STARCOIN_STDLIB_ADDR_NAME: &str = "starcoin_framework";
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
        PackageCommand::Errmap(c) => c.execute(move_args.package_path, move_args.build_config),
        PackageCommand::Prove(c) => c.execute(move_args.package_path, move_args.build_config),
        PackageCommand::Coverage(c) => c.execute(move_args.package_path, move_args.build_config),
        // XXX FIXME YSG
        PackageCommand::Test(c) => {
            natives::configure_for_unit_test();
            extended_checks::configure_extended_checks_for_unit_test();

            let mut build_config = move_args.build_config.clone();
            build_config
                .compiler_config
                .known_attributes
                .clone_from(extended_checks::get_all_attribute_names());
            c.execute(
                move_args.package_path,
                build_config,
                natives,
                starcoin_test_feature_flags_genesis(),
                None,
            )
        }
        PackageCommand::Disassemble(c) => c.execute(move_args.package_path, move_args.build_config),
    }
}
