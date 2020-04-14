// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CommandAction, EmptyOpt, FnCommandAction, NoneAction};
use anyhow::Result;
use clap::{App, ArgMatches};
use serde::export::PhantomData;
use std::collections::HashMap;
use std::sync::Arc;
use structopt::StructOpt;

pub(crate) trait CommandExec<State, GlobalOpt>
where
    GlobalOpt: StructOpt + 'static,
    State: 'static,
{
    fn exec(
        &mut self,
        state: Arc<State>,
        global_opt: Arc<GlobalOpt>,
        arg_matches: &ArgMatches<'_>,
    ) -> Result<()>;

    fn get_app(&mut self) -> &mut App<'static, 'static>;
}

pub struct ExecContext<State, GlobalOpt, Opt>
where
    State: 'static,
    GlobalOpt: StructOpt + 'static,
    Opt: StructOpt + 'static,
{
    state: Arc<State>,
    global_opt: Arc<GlobalOpt>,
    opt: Arc<Opt>,
}

impl<State, GlobalOpt, Opt> ExecContext<State, GlobalOpt, Opt>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
{
    pub fn new(state: Arc<State>, global_opt: Arc<GlobalOpt>, opt: Arc<Opt>) -> Self {
        Self {
            state,
            global_opt,
            opt,
        }
    }
    pub fn global_opt(&self) -> &GlobalOpt {
        self.global_opt.as_ref()
    }

    pub fn state(&self) -> &State {
        self.state.as_ref()
    }

    pub fn opt(&self) -> &Opt {
        self.opt.as_ref()
    }
}

pub struct Command<State, GlobalOpt, Opt, Action>
where
    GlobalOpt: StructOpt + 'static,
    State: 'static,
    Opt: StructOpt + 'static,
    Action: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt> + 'static,
{
    app: App<'static, 'static>,
    action: Option<Action>,
    subcommands: HashMap<String, Box<dyn CommandExec<State, GlobalOpt>>>,
    global_opt: PhantomData<GlobalOpt>,
    opt_type: PhantomData<Opt>,
}

impl<State, GlobalOpt> Command<State, GlobalOpt, EmptyOpt, NoneAction<State, GlobalOpt>>
where
    GlobalOpt: StructOpt,
{
    pub fn with_name(name: &str) -> Self {
        Self {
            app: App::new(name),
            action: None,
            subcommands: HashMap::new(),
            global_opt: PhantomData,
            opt_type: PhantomData,
        }
    }
}

impl<State, GlobalOpt, Opt> Command<State, GlobalOpt, Opt, FnCommandAction<State, GlobalOpt, Opt>>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
{
    pub fn with_action_fn<A>(action: A) -> Self
    where
        A: Fn(&ExecContext<State, GlobalOpt, Opt>) -> Result<()> + 'static,
    {
        Self {
            app: Opt::clap(),
            action: Some(FnCommandAction::new(action)),
            subcommands: HashMap::new(),
            global_opt: PhantomData,
            opt_type: PhantomData,
        }
    }
}

impl<State, GlobalOpt, Opt, Action> Command<State, GlobalOpt, Opt, Action>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
    Action: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt>,
{
    pub fn new() -> Self {
        Self {
            app: Opt::clap(),
            action: None,
            subcommands: HashMap::new(),
            global_opt: PhantomData,
            opt_type: PhantomData,
        }
    }

    pub fn with_action(action: Action) -> Self {
        Self {
            app: Opt::clap(),
            action: Some(action),
            subcommands: HashMap::new(),
            global_opt: PhantomData,
            opt_type: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        self.app.get_name()
    }

    pub fn app(&self) -> &App<'static, 'static> {
        &self.app
    }

    pub fn subcommand<SubOpt, SubAction, CMD>(mut self, subcommand: CMD) -> Self
    where
        SubOpt: StructOpt + 'static,
        SubAction: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = SubOpt> + 'static,
        CMD: Into<Command<State, GlobalOpt, SubOpt, SubAction>> + 'static,
    {
        let subcommand = subcommand.into();
        let name = subcommand.name();
        if self.subcommands.contains_key(name) {
            panic!("Subcommand with name {} exist.", name);
        }
        let order = self.subcommands.len();
        self.app = self
            .app
            .subcommand(subcommand.app().clone().display_order(order));
        self.subcommands
            .insert(name.to_string(), Box::new(subcommand));
        self
    }

    pub fn has_subcommand(&self) -> bool {
        !self.subcommands.is_empty()
    }

    fn exec_action(&mut self, ctx: &ExecContext<State, GlobalOpt, Opt>) -> Result<()> {
        match &self.action {
            Some(action) => {
                action.run(ctx)?;
            }
            None => {
                self.app.print_help()?;
            }
        }
        Ok(())
    }
}

impl<State, GlobalOpt, Opt, Action> CommandExec<State, GlobalOpt>
    for Command<State, GlobalOpt, Opt, Action>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
    Action: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt>,
{
    fn exec(
        &mut self,
        state: Arc<State>,
        global_opt: Arc<GlobalOpt>,
        arg_matches: &ArgMatches<'_>,
    ) -> Result<()> {
        let opt = Arc::new(Opt::from_clap(arg_matches));
        let ctx = ExecContext::new(state, global_opt, opt);
        if self.has_subcommand() {
            let (subcmd_name, subcmd_matches) = arg_matches.subcommand();
            match subcmd_name {
                "" => {
                    self.exec_action(&ctx)?;
                }
                subcmd_name => {
                    let subcmd = self.subcommands.get_mut(subcmd_name);
                    match (subcmd, subcmd_matches) {
                        (Some(subcmd), Some(subcmd_matches)) => {
                            subcmd.exec(ctx.state, ctx.global_opt, subcmd_matches)?;
                        }
                        _ => {
                            println!("Can not find subcomamnd: {}", subcmd_name);
                            self.exec_action(&ctx)?;
                        }
                    }
                }
            }
        } else {
            self.exec_action(&ctx)?;
        }
        Ok(())
    }

    fn get_app(&mut self) -> &mut App<'static, 'static> {
        &mut self.app
    }
}

impl<C, State, GlobalOpt, Opt> From<C> for Command<C::State, C::GlobalOpt, C::Opt, C>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
    C: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt> + 'static,
{
    fn from(action: C) -> Self {
        Command::with_action(action)
    }
}
