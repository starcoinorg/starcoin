// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CommandAction, EmptyOpt, FnCommandAction, NoneAction};
use anyhow::Result;
use clap::{App, ArgMatches};
use serde_json::Value;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use structopt::StructOpt;

pub(crate) enum HistoryOp {
    Skip,
    Record,
}

pub(crate) trait CommandExec<State, GlobalOpt>
where
    GlobalOpt: StructOpt + 'static,
    State: 'static,
{
    /// return HistoryOp and execute result value.
    // return HistoryOp as execute result is not a good design, may been change in the future.
    fn exec(
        &mut self,
        state: Arc<State>,
        global_opt: Arc<GlobalOpt>,
        arg_matches: &ArgMatches<'_>,
    ) -> Result<(HistoryOp, Value)>;

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

pub struct Command<State, GlobalOpt, Opt, ReturnItem, Action>
where
    GlobalOpt: StructOpt + 'static,
    State: 'static,
    Opt: StructOpt + 'static,
    ReturnItem: serde::Serialize + 'static,
    Action: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt, ReturnItem = ReturnItem>
        + 'static,
{
    app: App<'static, 'static>,
    action: Option<Action>,
    subcommands: HashMap<String, Box<dyn CommandExec<State, GlobalOpt>>>,
    global_opt: PhantomData<GlobalOpt>,
    opt_type: PhantomData<Opt>,
}

impl<State, GlobalOpt> Command<State, GlobalOpt, EmptyOpt, (), NoneAction<State, GlobalOpt>>
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

impl<State, GlobalOpt, Opt, ReturnItem>
    Command<State, GlobalOpt, Opt, ReturnItem, FnCommandAction<State, GlobalOpt, Opt, ReturnItem>>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
    ReturnItem: serde::Serialize,
{
    pub fn with_action_fn<A>(action: A) -> Self
    where
        A: Fn(&ExecContext<State, GlobalOpt, Opt>) -> Result<ReturnItem> + 'static,
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

impl<State, GlobalOpt, Opt, ReturnItem, Action> Command<State, GlobalOpt, Opt, ReturnItem, Action>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
    ReturnItem: serde::Serialize + 'static,
    Action: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt, ReturnItem = ReturnItem>,
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

    pub fn with_about(mut self, about: &'static str) -> Self {
        self.app = self.app.about(about);
        self
    }

    pub fn name(&self) -> &str {
        self.app.get_name()
    }

    pub fn app(&self) -> &App<'static, 'static> {
        &self.app
    }

    pub fn subcommand<SubOpt, SubReturnItem, SubAction, CMD>(mut self, subcommand: CMD) -> Self
    where
        SubOpt: StructOpt + 'static,
        SubReturnItem: serde::Serialize + 'static,
        SubAction: CommandAction<
                State = State,
                GlobalOpt = GlobalOpt,
                Opt = SubOpt,
                ReturnItem = SubReturnItem,
            > + 'static,
        CMD: Into<Command<State, GlobalOpt, SubOpt, SubReturnItem, SubAction>> + 'static,
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

    pub fn help_message(&mut self) -> String {
        let mut help_message = vec![];
        self.app
            .write_long_help(&mut help_message)
            .expect("format help message fail.");
        String::from_utf8(help_message).expect("help message should utf8")
    }

    fn exec_action(
        &mut self,
        ctx: &ExecContext<State, GlobalOpt, Opt>,
    ) -> Result<(HistoryOp, Value)> {
        match &self.action {
            Some(action) => {
                let skip_history = action.skip_history(ctx);
                let skip_history_op = if skip_history {
                    HistoryOp::Skip
                } else {
                    HistoryOp::Record
                };
                let ret = action.run(ctx)?;
                Ok((skip_history_op, serde_json::to_value(ret)?))
            }
            None => Err(anyhow::Error::msg(self.help_message())),
        }
    }
}

impl<State, GlobalOpt, Opt, ReturnItem, Action> Default
    for Command<State, GlobalOpt, Opt, ReturnItem, Action>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
    ReturnItem: serde::Serialize + 'static,
    Action: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt, ReturnItem = ReturnItem>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<State, GlobalOpt, Opt, ReturnItem, Action> CommandExec<State, GlobalOpt>
    for Command<State, GlobalOpt, Opt, ReturnItem, Action>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
    ReturnItem: serde::Serialize + 'static,
    Action: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt, ReturnItem = ReturnItem>,
{
    fn exec(
        &mut self,
        state: Arc<State>,
        global_opt: Arc<GlobalOpt>,
        arg_matches: &ArgMatches<'_>,
    ) -> Result<(HistoryOp, Value)> {
        let opt = Arc::new(Opt::from_clap(arg_matches));
        let ctx = ExecContext::new(state, global_opt, opt);
        let value = if self.has_subcommand() {
            let (subcmd_name, subcmd_matches) = arg_matches.subcommand();
            match subcmd_name {
                "" => self.exec_action(&ctx)?,
                subcmd_name => {
                    let subcmd = self.subcommands.get_mut(subcmd_name);
                    match (subcmd, subcmd_matches) {
                        (Some(subcmd), Some(subcmd_matches)) => {
                            subcmd.exec(ctx.state, ctx.global_opt, subcmd_matches)?
                        }
                        _ => unreachable!(
                            "this should not happen, because sub cmd has check by clip."
                        ),
                    }
                }
            }
        } else {
            self.exec_action(&ctx)?
        };
        Ok(value)
    }

    fn get_app(&mut self) -> &mut App<'static, 'static> {
        &mut self.app
    }
}

impl<C, State, GlobalOpt, Opt, ReturnItem> From<C>
    for Command<C::State, C::GlobalOpt, C::Opt, C::ReturnItem, C>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
    ReturnItem: serde::Serialize,
    C: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt, ReturnItem = ReturnItem>,
{
    fn from(action: C) -> Self {
        Command::with_action(action)
    }
}
