use move_binary_format::file_format_common::VERSION_3;
use move_binary_format::CompiledModule;
use move_cli::sandbox::utils::PackageContext;
use move_cli::Move;
use move_compiler::compiled_unit::{CompiledUnit, NamedCompiledModule};
use move_core_types::language_storage::TypeTag;
use move_core_types::transaction_argument::{convert_txn_args, TransactionArgument};
use starcoin_move_compiler::bytecode_transpose::ModuleBytecodeDowgrader;
use starcoin_types::transaction::parse_transaction_argument;
use starcoin_vm_types::language_storage::FunctionId;
use starcoin_vm_types::parser::parse_type_tag;
use starcoin_vm_types::transaction::{Module, Package, ScriptFunction};
use std::path::PathBuf;
use structopt::StructOpt;

pub const DEFAULT_RELEASE_DIR: &str = "release";

#[derive(StructOpt)]
pub struct Releasement {
    #[structopt(name = "move-version", long = "move-version", default_value="3", possible_values=&["3", "4"])]
    /// specify the move lang version for the release.
    /// currently, only v3, v4 are supported.
    language_version: u8,

    #[structopt(name="release-dir", long, parse(from_os_str), default_value=DEFAULT_RELEASE_DIR)]
    /// dir to store released blob
    release_dir: PathBuf,

    #[structopt(long = "function", name = "script-function")]
    /// init script function to execute, example: 0x123::MyScripts::init_script
    init_script: Option<FunctionId>,

    #[structopt(
    short = "t",
    long = "type_tag",
    name = "type-tag",
    parse(try_from_str = parse_type_tag)
    )]
    /// type tags for the init script function
    type_tags: Option<Vec<TypeTag>>,

    #[structopt(long = "arg", name = "transaction-args", parse(try_from_str = parse_transaction_argument))]
    /// args for the init script function
    args: Option<Vec<TransactionArgument>>,
}

pub fn handle_release(
    move_args: &Move,
    Releasement {
        language_version,
        mut release_dir,
        init_script,
        type_tags,
        args,
    }: Releasement,
) -> anyhow::Result<()> {
    let mut ms = vec![];
    let pkg_ctx = PackageContext::new(&move_args.package_path, &move_args.build_config)?;
    let pkg = pkg_ctx.package();
    let pkg_version = move_args
        .build_config
        .clone()
        .resolution_graph_for_package(&move_args.package_path)
        .unwrap()
        .root_package
        .package
        .version;
    let pkg_name = pkg.compiled_package_info.package_name.as_str();
    println!("Packaging Modules:");
    for m in pkg.modules()? {
        let m = module(&m.unit)?;
        println!("\t {}", m.self_id());
        let code = if language_version as u32 == VERSION_3 {
            ModuleBytecodeDowgrader::to_v3(m)?
        } else {
            let mut data = vec![];
            m.serialize(&mut data)?;
            data
        };
        ms.push(Module::new(code));
    }
    let init_script = match &init_script {
        Some(script) => {
            let type_tags = type_tags.unwrap_or_default();
            let args = args.unwrap_or_default();
            let script_function = script.clone();
            Some(ScriptFunction::new(
                script_function.module,
                script_function.function,
                type_tags,
                convert_txn_args(&args),
            ))
        }
        None => None,
    };

    let p = Package::new(ms, init_script)?;
    let package_bytes = bcs_ext::to_bytes(&p)?;
    let release_path = {
        std::fs::create_dir_all(&release_dir)?;
        release_dir.push(format!(
            "{}.v{}.{}.{}.blob",
            pkg_name, pkg_version.0, pkg_version.1, pkg_version.2
        ));
        release_dir
    };
    std::fs::write(&release_path, package_bytes)?;
    println!("Release done: {}", release_path.display());
    Ok(())
}

pub fn module(unit: &CompiledUnit) -> anyhow::Result<&CompiledModule> {
    match unit {
        CompiledUnit::Module(NamedCompiledModule { module, .. }) => Ok(module),
        _ => anyhow::bail!("Found script in modules -- this shouldn't happen"),
    }
}
