// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use scmd::{CmdContext, Command, CommandAction, ExecContext};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hello_app")]
struct GlobalOpts {
    #[structopt(short = "c", default_value = "0")]
    counter: usize,
    #[structopt(short = "r")]
    required: String,
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
    pub fn new(init: usize) -> Self {
        Self(AtomicUsize::new(init))
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

struct BetaCommand;

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

#[derive(Debug, StructOpt)]
#[structopt(name = "test")]
struct TestOpts {
    #[structopt(short = "d")]
    debug: bool,
}

fn main() -> Result<()> {
    let context = CmdContext::<Counter, GlobalOpts>::with_default_action(
        |global_opt| -> Result<Counter> { Ok(Counter::new(global_opt.counter)) },
        |_app, _opt, _state| {
            let running = Arc::new(AtomicBool::new(true));
            let r = running.clone();
            ctrlc::set_handler(move || {
                r.store(false, Ordering::SeqCst);
            })
            .expect("Error setting Ctrl-C handler");
            println!("Waiting for Ctrl-C...");
            while running.load(Ordering::SeqCst) {}
            println!("Got it! Exiting...");
        },
        |_app, _opt, _state| println!("Start a console:"),
        |_app, _opt, _state| println!("good bye."),
    );
    context
        .command(
            Command::with_name("alpha").subcommand(Command::with_action_fn(
                |ctx: &ExecContext<Counter, GlobalOpts, AlphaSub1Opts>| -> Result<()> {
                    println!(
                        "hello global_opts:{:?} {:?} state:{:?}",
                        ctx.global_opt(),
                        ctx.opt(),
                        ctx.state().get()
                    );
                    Ok(())
                },
            )),
        )
        .command(BetaCommand.into_cmd().subcommand(BetaSub1Command {}))
        .command(Command::with_action_fn(
            |ctx: &ExecContext<Counter, GlobalOpts, TestOpts>| -> Result<()> {
                println!(
                    "hello test global_opts:{:?} {:?} state:{:?}",
                    ctx.global_opt(),
                    ctx.opt(),
                    ctx.state().get()
                );
                Ok(())
            },
        ))
        .exec()?;
    Ok(())
}
