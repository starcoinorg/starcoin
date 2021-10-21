use anyhow::Result;
use move_cli::package;
use move_package;
use std::path::PathBuf;
use structopt::StructOpt;
#[derive(StructOpt)]
pub struct Commands {
    /// Path to package. If none is supplied the current directory will be used.
    #[structopt(long = "path", short = "p", global = true, parse(from_os_str))]
    path: Option<PathBuf>,

    #[structopt(flatten)]
    config: move_package::BuildConfig,

    #[structopt(flatten)]
    cmd: package::cli::PackageCommand,
}
fn main() -> Result<()> {
    let command: Commands = Commands::from_args();

    package::cli::handle_package_commands(&command.path, command.config.clone(), &command.cmd)?;
    Ok(())
}
