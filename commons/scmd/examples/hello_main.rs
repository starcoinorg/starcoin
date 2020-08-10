// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::Rng;
use scmd::{CmdContext, Command, CommandAction, ExecContext};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hello")]
struct GlobalOpts {
    #[structopt(short = "c", default_value = "0")]
    counter: usize,
    #[structopt(short = "r")]
    required: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Address {
    pub city: String,
    pub zip: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    pub index: usize,
    pub name: String,
    pub age: u32,
    pub address: Address,
}

impl User {
    pub fn random(index: usize) -> Self {
        let mut rng = rand::thread_rng();
        let name: String = rng.sample_iter(&Alphanumeric).take(10).collect();
        let age: u32 = rng.gen();
        let city = rng.sample_iter(&Alphanumeric).take(5).collect();
        let zip = rng.gen_range(10000, 99999);
        let address = Address { city, zip };
        Self {
            index,
            name,
            age,
            address,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "list")]
struct ListOpts {
    #[structopt(long, short = "c", default_value = "5")]
    count: usize,
}

struct ListCommand;

impl CommandAction for ListCommand {
    type State = Counter;
    type GlobalOpt = GlobalOpts;
    type Opt = ListOpts;
    type ReturnItem = Vec<User>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let count = ctx.opt().count;
        let mut users = vec![];
        for i in 0..count {
            users.push(User::random(i));
        }
        Ok(users)
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "show")]
struct ShowOpts {
    #[structopt(long, default_value = "0")]
    index: usize,
}

struct ShowCommand;

impl CommandAction for ShowCommand {
    type State = Counter;
    type GlobalOpt = GlobalOpts;
    type Opt = ShowOpts;
    type ReturnItem = User;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        Ok(User::random(ctx.opt().index))
    }
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
    type ReturnItem = ();

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
    type ReturnItem = ();

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

pub(crate) fn init_context() -> CmdContext<Counter, GlobalOpts> {
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
    );
    let context = context.with_console_support_default();
    context
        .command(ListCommand)
        .command(ShowCommand)
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
}

fn main() -> Result<()> {
    let context = init_context();
    context.exec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_command_with_args() -> Result<()> {
        let context = init_context();
        let count = 10;
        let result = context.exec_with_args::<Vec<User>>(vec![
            "hello",
            "-r",
            "test_required",
            "list",
            "-c",
            format!("{}", count).as_str(),
        ])?;
        assert_eq!(result.len(), count);
        Ok(())
    }
}
