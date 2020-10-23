// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use chrono::Local;
use env_logger::{self, fmt::Color};
use log::Level;
use std::io::Write;
use std::path::Path;
use structopt::StructOpt;

pub mod bench {
    pub use libra_x::bench::*;
}

pub mod check {
    pub use libra_x::check::*;
}
pub mod clippy {
    pub use libra_x::clippy::*;
}
pub mod config {
    pub use libra_x::config::*;
}
pub mod context {
    pub use libra_x::context::*;
}
pub mod diff_summary {
    pub use libra_x::diff_summary::*;
}
pub mod fix {
    pub use libra_x::fix::*;
}
pub mod fmt {
    pub use libra_x::fmt::*;
}
pub mod generate_summaries {
    pub use libra_x::generate_summaries::*;
}
pub mod installer {
    pub use libra_x::installer::*;
}
#[cfg(not(windows))]
pub mod lint {
    pub use libra_x::lint::*;
}
pub mod test;

pub mod cargo;

pub mod tools {
    pub use libra_x::tools::*;
}

pub mod utils {
    pub use libra_x::utils::*;
}

pub fn project_root() -> &'static Path {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}

type Result<T> = anyhow::Result<T>;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "bench")]
    /// Run `cargo bench`
    Bench(bench::Args),
    #[structopt(name = "check")]
    /// Run `cargo check`
    Check(check::Args),
    #[structopt(name = "clippy")]
    /// Run `cargo clippy`
    Clippy(clippy::Args),
    #[structopt(name = "fix")]
    /// Run `cargo fix`
    Fix(fix::Args),
    #[structopt(name = "fmt")]
    /// Run `cargo fmt`
    Fmt(fmt::Args),
    #[structopt(name = "test")]
    /// Run tests
    Test(test::Args),
    #[structopt(name = "tools")]
    /// Run tests
    Tools(tools::Args),
    #[structopt(name = "lint")]
    /// Run lints
    Lint(lint::Args),
    #[structopt(name = "generate-summaries")]
    /// Generate build summaries for important subsets
    GenerateSummaries(generate_summaries::Args),
    #[structopt(name = "diff-summary")]
    /// Diff build summaries for important subsets
    DiffSummary(diff_summary::Args),
}

fn main() -> Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let color = match record.level() {
                Level::Warn => Color::Yellow,
                Level::Error => Color::Red,
                _ => Color::Green,
            };

            let mut level_style = buf.style();
            level_style.set_color(color).set_bold(true);

            writeln!(
                buf,
                "{:>12} [{}] - {}",
                level_style.value(record.level()),
                Local::now().format("%T%.3f"),
                record.args()
            )
        })
        .init();

    let args = Args::from_args();
    let xctx = context::XContext::new()?;
    // let config = libra_x::config::Config::from_file("./x.toml")?;
    // let xctx = context::XContext::with_config(config);
    // dbg!(xctx.config());
    match args.cmd {
        Command::Tools(args) => tools::run(args, xctx),
        Command::Test(args) => test::run(args, xctx),
        Command::Check(args) => check::run(args, xctx),
        Command::Clippy(args) => clippy::run(args, xctx),
        Command::Fix(args) => fix::run(args, xctx),
        Command::Fmt(args) => fmt::run(args, xctx),
        Command::Bench(args) => bench::run(args, xctx),
        Command::Lint(args) => lint::run(args, xctx),
        Command::GenerateSummaries(args) => generate_summaries::run(args, xctx),
        Command::DiffSummary(args) => diff_summary::run(args, xctx),
    }
}
