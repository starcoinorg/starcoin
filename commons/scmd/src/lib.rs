// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::{App, ArgMatches};
use serde::export::PhantomData;
use std::collections::HashMap;
use structopt::StructOpt;

pub trait CommandRunnable {
    fn run(&self, arg_matches: &ArgMatches<'_>) -> Result<()>;
}

pub struct CmdContext<'a, 'b> {
    app: App<'a, 'b>,
    commands: HashMap<String, Box<dyn CommandRunnable>>,
}

impl<'a, 'b> CmdContext<'a, 'b> {
    pub fn new(app_name: &str) -> Self {
        Self {
            app: App::new(app_name),
            commands: HashMap::new(),
        }
    }

    pub fn with_command<Arg>(mut self, action: Box<dyn Fn(Arg) -> Result<()>>) -> Self
    where
        Arg: StructOpt + 'static,
    {
        let command = Command::new(action);
        let name = command.name().to_string();
        self.app = self.app.subcommand(command.app().clone());
        self.commands.insert(name, Box::new(command));
        self
    }

    pub fn execute(self) -> Result<()> {
        let matches = self.app.get_matches();

        let (cmd, arg_matches) = matches.subcommand();
        let cmd = self.commands.get(cmd);
        match (cmd, arg_matches) {
            (Some(cmd), Some(arg_matches)) => {
                cmd.run(arg_matches)?;
            }
            _ => {}
        };
        Ok(())
    }
}

pub struct Command<'a, 'b, Arg>
where
    Arg: StructOpt,
{
    pub app: App<'a, 'b>,
    pub action: Box<dyn Fn(Arg) -> Result<()>>,
    pub arg: PhantomData<Arg>,
}

impl<'a, 'b, Arg> Command<'a, 'b, Arg>
where
    Arg: StructOpt,
{
    pub fn new(action: Box<dyn Fn(Arg) -> Result<()>>) -> Self {
        Self {
            app: Arg::clap(),
            action,
            arg: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        self.app.get_name()
    }

    pub fn app(&self) -> &App<'a, 'b> {
        &self.app
    }
}

impl<'a, 'b, Arg> CommandRunnable for Command<'a, 'b, Arg>
where
    Arg: StructOpt,
{
    fn run(&self, arg_matches: &ArgMatches<'_>) -> Result<()> {
        let arg = Arg::from_clap(arg_matches);
        self.action.as_ref()(arg)
        //Ok(())
    }
}
