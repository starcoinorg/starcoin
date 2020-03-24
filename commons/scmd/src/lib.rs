// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::{App, ArgMatches, SubCommand};
use rustyline::{config::CompletionType, error::ReadlineError, Config, Editor};
use serde::export::PhantomData;
use std::collections::HashMap;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Debug, Clone, Default, StructOpt)]
pub struct EmptyOpt {}

pub trait CommandAction {
    type State;
    type GlobalOpt: StructOpt;
    type Opt: StructOpt;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()>;

    fn into_cmd(self) -> Command<Self::State, Self::GlobalOpt, Self::Opt, Self>
    where
        Self: std::marker::Sized,
    {
        self.into()
    }
}

pub struct FnCommandAction<State, GlobalOpt, Opt>
where
    State: 'static,
    GlobalOpt: StructOpt + 'static,
    Opt: StructOpt + 'static,
{
    action: Box<dyn Fn(&ExecContext<State, GlobalOpt, Opt>) -> Result<()>>,
}

impl<State, GlobalOpt, Opt> FnCommandAction<State, GlobalOpt, Opt>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
{
    pub fn new(action: Box<dyn Fn(&ExecContext<State, GlobalOpt, Opt>) -> Result<()>>) -> Self {
        Self { action }
    }
}

impl<State, GlobalOpt, Opt> CommandAction for FnCommandAction<State, GlobalOpt, Opt>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
{
    type State = State;
    type GlobalOpt = GlobalOpt;
    type Opt = Opt;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        self.action.as_ref()(ctx)
    }
}

pub struct NoneAction<State, GlobalOpt>
where
    State: 'static,
    GlobalOpt: StructOpt + 'static,
{
    state_type: PhantomData<State>,
    global_opt_type: PhantomData<GlobalOpt>,
}

impl<State, GlobalOpt> CommandAction for NoneAction<State, GlobalOpt>
where
    GlobalOpt: StructOpt,
{
    type State = State;
    type GlobalOpt = GlobalOpt;
    type Opt = EmptyOpt;

    fn run(&self, _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        Ok(())
    }
}

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

pub struct CmdContext<State, GlobalOpt>
where
    State: 'static,
    GlobalOpt: StructOpt + 'static,
{
    app: App<'static, 'static>,
    commands: HashMap<String, Box<dyn CommandExec<State, GlobalOpt>>>,
    state: Arc<State>,
}

impl<State, GlobalOpt> CmdContext<State, GlobalOpt>
where
    GlobalOpt: StructOpt,
{
    pub fn new(state: State) -> Self {
        //insert console command
        let mut app = GlobalOpt::clap();
        app = app.subcommand(
            SubCommand::with_name("console").help("Start an interactive command console"),
        );
        Self {
            app,
            commands: HashMap::new(),
            state: Arc::new(state),
        }
    }

    pub fn add_command<Opt, Action>(
        mut self,
        command: Command<State, GlobalOpt, Opt, Action>,
    ) -> Self
    where
        Opt: StructOpt + 'static,
        Action: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt> + 'static,
    {
        let name = command.name().to_string();
        self.app = self.app.subcommand(command.app().clone());
        self.commands.insert(name, Box::new(command));
        self
    }

    pub fn print_help(&mut self) {
        self.app.print_help().expect("print help should success.");
    }

    pub fn exec(mut self) -> Result<()> {
        let matches = match self
            .app
            .get_matches_from_safe_borrow(&mut std::env::args_os())
        {
            Ok(matches) => matches,
            Err(e) => {
                //println!("Match err: {:?}", e.kind);
                println!("{}", e.message);
                return Ok(());
            }
        };

        let global_opt = Arc::new(GlobalOpt::from_clap(&matches));
        let (cmd_name, arg_matches) = matches.subcommand();
        match cmd_name {
            "console" => {
                self.console_inner(global_opt);
            }
            "" => {
                self.print_help();
            }
            cmd_name => {
                let cmd = self.commands.get_mut(cmd_name);
                match (cmd, arg_matches) {
                    (Some(cmd), Some(arg_matches)) => {
                        cmd.exec(self.state.clone(), global_opt, arg_matches)?;
                    }
                    _ => {
                        println!("Unknown command: {:?}", cmd_name);
                        self.print_help();
                    }
                };
            }
        }

        Ok(())
    }

    fn console_inner(mut self, global_opt: Arc<GlobalOpt>) {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .auto_add_history(true)
            .build();
        // insert quit command
        self.app = self.app.subcommand(
            SubCommand::with_name("quit")
                .aliases(&["exit", "q!"])
                .help("Quit from console."),
        );
        let app_name = self.app.get_name().to_string();
        let mut rl = Editor::<()>::with_config(config);
        loop {
            let readline = rl.readline("starcoin% ");
            match readline {
                Ok(line) => {
                    let mut params: Vec<&str> = line.trim().split(' ').map(str::trim).collect();
                    // insert the app name to first, for match env args style.
                    params.insert(0, app_name.as_str());
                    let arg_matches = match self.app.get_matches_from_safe_borrow(params) {
                        Ok(arg_matches) => arg_matches,
                        Err(e) => {
                            //println!("match err: {:?}", e.kind);
                            println!("{}", e.message);
                            continue;
                        }
                    };
                    let (cmd_name, arg_matches) = arg_matches.subcommand();
                    match cmd_name {
                        "quit" => break,
                        "console" => continue,
                        "" => continue,
                        cmd_name => {
                            let cmd = self.commands.get_mut(cmd_name);
                            match (cmd, arg_matches) {
                                (Some(cmd), Some(arg_matches)) => {
                                    match cmd.exec(
                                        self.state.clone(),
                                        global_opt.clone(),
                                        arg_matches,
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => println!("Execute error:{:?}", e),
                                    };
                                }
                                _ => println!("Unknown command: {:?}", cmd_name),
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }

    pub fn console(mut self) -> Result<()> {
        let matches = match self
            .app
            .get_matches_from_safe_borrow(&mut std::env::args_os())
        {
            Ok(matches) => matches,
            Err(e) => {
                //println!("Match err: {:?}", e.kind);
                println!("{}", e.message);
                return Ok(());
            }
        };
        let global_opt = Arc::new(GlobalOpt::from_clap(&matches));
        self.console_inner(global_opt);
        Ok(())
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
    pub fn with_action_fn(
        action: Box<dyn Fn(&ExecContext<State, GlobalOpt, Opt>) -> Result<()>>,
    ) -> Self {
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

    pub fn subcommand<SubOpt, SubAction>(
        mut self,
        subcommand: Command<State, GlobalOpt, SubOpt, SubAction>,
    ) -> Self
    where
        SubOpt: StructOpt + 'static,
        SubAction: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = SubOpt>,
    {
        self.app = self.app.subcommand(subcommand.app().clone());
        self.subcommands
            .insert(subcommand.name().to_string(), Box::new(subcommand));
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
            let subcmd = self.subcommands.get_mut(subcmd_name);
            match (subcmd, subcmd_matches) {
                (Some(subcmd), Some(subcmd_matches)) => {
                    subcmd.exec(ctx.state, ctx.global_opt, subcmd_matches)?;
                }
                _ => {
                    self.exec_action(&ctx)?;
                }
            }
        } else {
            self.exec_action(&ctx)?;
        }
        Ok(())
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
