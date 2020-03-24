// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use scmd::{CmdContext, Command, CommandAction, ExecContext};
use std::sync::atomic::{AtomicUsize, Ordering};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hello_app")]
struct GlobalOpts {
    #[structopt(short = "t")]
    test: bool,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "alpha")]
struct AlphaOpts {
    #[structopt(short = "n", default_value = "alpha_default")]
    name: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "alpha_sub1")]
struct AlphaSub1Opts {
    #[structopt(short = "n", default_value = "alpha_sub1_default")]
    name: String,
}

struct Counter(AtomicUsize);

impl Counter {
    pub fn new() -> Self {
        Self(AtomicUsize::default())
    }

    pub fn incr(&self) -> usize {
        self.0.fetch_add(1, Ordering::Relaxed)
    }

    pub fn get(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "beta")]
struct BetaOpts {
    #[structopt(short = "n", default_value = "beta_default")]
    name: String,
}

struct BetaCommand {}

impl CommandAction for BetaCommand {
    type State = Counter;
    type GlobalOpt = GlobalOpts;
    type Opt = BetaOpts;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        println!(
            "BetaCommand hello global_opts:{:?} {:?} state:{:?}",
            ctx.global_opt(),
            ctx.opt(),
            ctx.state().get()
        );
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "beta_sub1")]
struct BetaSub1Opts {
    #[structopt(short = "n", default_value = "beta_default")]
    name: String,
}

struct BetaSub1Command {}

impl CommandAction for BetaSub1Command {
    type State = Counter;
    type GlobalOpt = GlobalOpts;
    type Opt = BetaSub1Opts;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        ctx.state().incr();
        println!(
            "BetaSub1Command hello global_opts:{:?} {:?} state:{:?}",
            ctx.global_opt(),
            ctx.opt(),
            ctx.state().get()
        );
        Ok(())
    }
}

fn main() -> Result<()> {
    let state = Counter::new();

    let context = CmdContext::new(state);
    context
        .add_command(
            Command::with_name("alpha").subcommand(Command::with_action_fn(Box::new(
                |ctx: &ExecContext<Counter, GlobalOpts, AlphaSub1Opts>| -> Result<()> {
                    println!(
                        "hello global_opts:{:?} {:?} state:{:?}",
                        ctx.global_opt(),
                        ctx.opt(),
                        ctx.state().get()
                    );
                    Ok(())
                },
            ))),
        )
        .add_command(
            BetaCommand {}
                .into_cmd()
                .subcommand(BetaSub1Command {}.into_cmd()),
        )
        .exec()?;
    Ok(())
}
