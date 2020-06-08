// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::CmdError;
use crate::{print_action_result, Command, CommandAction, CommandExec, OutputFormat};
use anyhow::Result;
use clap::{crate_authors, crate_version, App, Arg, SubCommand};
use git_version::git_version;
use lazy_static::lazy_static;
use rustyline::{config::CompletionType, error::ReadlineError, Config, Editor};
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::Arc;
use structopt::StructOpt;

static OUTPUT_FORMAT_ARG: &str = "output-format";
static VERSION: &str = crate_version!();
static GIT_VERSION: &str = git_version!();
lazy_static! {
    static ref LONG_VERSION: String = format!("{} (build:{})", VERSION, GIT_VERSION);
}
pub struct CmdContext<State, GlobalOpt>
where
    State: 'static,
    GlobalOpt: StructOpt + 'static,
{
    app: App<'static, 'static>,
    commands: HashMap<String, Box<dyn CommandExec<State, GlobalOpt>>>,
    default_action: Box<dyn FnOnce(App, GlobalOpt, State)>,
    state_initializer: Box<dyn FnOnce(&GlobalOpt) -> Result<State>>,
    console_start_action: Box<dyn FnOnce(&App, Arc<GlobalOpt>, Arc<State>)>,
    console_quit_action: Box<dyn FnOnce(App, GlobalOpt, State)>,
}

impl<State, GlobalOpt> CmdContext<State, GlobalOpt>
where
    State: 'static,
    GlobalOpt: StructOpt + 'static,
{
    /// Init new CmdContext with State
    pub fn with_state(state: State) -> Self {
        Self::with_initializer(|_opts| Ok(state))
    }

    /// Init new CmdContext with state_initializer
    /// default action is print help.
    pub fn with_initializer<I>(state_initializer: I) -> Self
    where
        I: FnOnce(&GlobalOpt) -> Result<State> + 'static,
    {
        Self::with_default_action(
            state_initializer,
            |mut app, _opt, _state| {
                app.print_long_help().expect("print help should success.");
            },
            |_app, _opt, _state| {},
            |_app, _opt, _state| {
                println!("quit.");
            },
        )
    }

    /// Init new CmdContext with state_initializer, and default_action, console_start_action, console_quit_action
    /// default_action executed when no subcommand is provided.
    /// console_start_action executed when start a console.
    /// console_quit_action executed when input quit subcommand at console.
    // note: D and Q's fn signature is same but is different type.
    pub fn with_default_action<I, D, S, Q>(
        state_initializer: I,
        default_action: D,
        console_start_action: S,
        console_quit_action: Q,
    ) -> Self
    where
        I: FnOnce(&GlobalOpt) -> Result<State> + 'static,
        D: FnOnce(App, GlobalOpt, State) + 'static,
        S: FnOnce(&App, Arc<GlobalOpt>, Arc<State>) + 'static,
        Q: FnOnce(App, GlobalOpt, State) + 'static,
    {
        //insert console command
        let mut app = GlobalOpt::clap();
        app = app
            .version(VERSION)
            .long_version(LONG_VERSION.as_str())
            .arg(
                Arg::with_name(OUTPUT_FORMAT_ARG)
                    .short("o")
                    .help("set output-format, support [json|table]")
                    .takes_value(true)
                    .default_value("table"),
            );
        app = Self::set_app_author(app);
        app = app.subcommand(
            SubCommand::with_name("console").help("Start an interactive command console"),
        );
        Self {
            app,
            commands: HashMap::new(),
            default_action: Box::new(default_action),
            state_initializer: Box::new(state_initializer),
            console_start_action: Box::new(console_start_action),
            console_quit_action: Box::new(console_quit_action),
        }
    }

    //remove this after clap upgrade
    //use of deprecated item 'std::sync::ONCE_INIT': the `new` function is now preferred
    #[allow(deprecated)]
    fn set_app_author<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app.author(crate_authors!("\n"))
    }

    pub fn command<Opt, ReturnItem, Action, CMD>(mut self, command: CMD) -> Self
    where
        Opt: StructOpt + 'static,
        ReturnItem: serde::Serialize + 'static,
        Action: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt, ReturnItem = ReturnItem>
            + 'static,
        CMD: Into<Command<State, GlobalOpt, Opt, ReturnItem, Action>> + 'static,
    {
        let command = command.into();
        let name = command.name();
        if self.commands.contains_key(name) {
            panic!("Command with name {} exist.", name);
        }
        let order = self.commands.len();
        self.app = self
            .app
            .subcommand(command.app().clone().display_order(order));
        self.commands.insert(name.to_string(), Box::new(command));
        self
    }

    pub fn print_help(&mut self) {
        self.app
            .print_long_help()
            .expect("print help should success.");
    }

    pub fn help_message(&mut self) -> String {
        Self::app_help_message(&mut self.app)
    }

    fn app_help_message(app: &mut App) -> String {
        let mut help_message = vec![];
        app.write_long_help(&mut help_message)
            .expect("format help message fail.");
        String::from_utf8(help_message).expect("help message should utf8")
    }

    /// Execute command by parse std::env::args_os() and print result.
    pub fn exec(self) {
        match self.exec_inner(&mut std::env::args_os()) {
            Err(e) => println!("init command context error: {}", e.to_string()),
            Ok((output_format, result)) => {
                if let Err(e) = print_action_result(output_format, result) {
                    println!("print result error: {}", e.to_string())
                }
            }
        }
    }

    /// Execute command by args and return Command execute ReturnItem
    pub fn exec_with_args<ReturnItem>(self, args: Vec<&str>) -> Result<ReturnItem>
    where
        ReturnItem: for<'de> serde::Deserialize<'de> + serde::Serialize + 'static,
    {
        let (_output_format, result) = self.exec_inner(args)?;
        let value = result?;
        serde_json::from_value(value).map_err(|e| e.into())
    }

    /// Execute command by the args.
    fn exec_inner<I, T>(mut self, iter: I) -> Result<(OutputFormat, Result<Value>)>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut app = self.app;
        let matches = app.get_matches_from_safe_borrow(iter)?;
        let output_format = matches
            .value_of(OUTPUT_FORMAT_ARG)
            .expect("output-format arg must exist")
            .parse()
            .expect("parse output-format must success.");

        let global_opt = GlobalOpt::from_clap(&matches);
        let state = (self.state_initializer)(&global_opt)?;

        let (cmd_name, arg_matches) = matches.subcommand();
        let default_action = self.default_action;
        let result = match cmd_name {
            "console" => {
                let start_action = self.console_start_action;
                let quit_action = self.console_quit_action;
                let commands = self.commands;
                Self::console_inner(app, global_opt, state, commands, start_action, quit_action);
                Ok(Value::Null)
            }
            "" => {
                default_action(app, global_opt, state);
                Ok(Value::Null)
            }
            cmd_name => {
                let cmd = self.commands.get_mut(cmd_name);
                match (cmd, arg_matches) {
                    (Some(cmd), Some(arg_matches)) => {
                        cmd.exec(Arc::new(state), Arc::new(global_opt), arg_matches)
                        //print_action_result(value, output_format)?;
                    }
                    _ => Err(CmdError::NeedHelp {
                        help: Self::app_help_message(&mut app),
                    }
                    .into()),
                }
            }
        };
        Ok((output_format, result))
    }

    fn console_inner(
        app: App,
        global_opt: GlobalOpt,
        state: State,
        mut commands: HashMap<String, Box<dyn CommandExec<State, GlobalOpt>>>,
        start_action: Box<dyn FnOnce(&App, Arc<GlobalOpt>, Arc<State>)>,
        quit_action: Box<dyn FnOnce(App, GlobalOpt, State)>,
    ) {
        //TODO support user custom config
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .auto_add_history(true)
            .build();
        //insert quit command
        let mut app = app
            .subcommand(
                SubCommand::with_name("version")
                    .help("Print app version.")
                    .display_order(997),
            )
            .subcommand(
                SubCommand::with_name("quit")
                    .aliases(&["exit", "q!"])
                    .help("Quit from console.")
                    .display_order(998),
            );
        let app_name = app.get_name().to_string();
        let global_opt = Arc::new(global_opt);
        let state = Arc::new(state);
        start_action(&app, global_opt.clone(), state.clone());
        let mut rl = Editor::<()>::with_config(config);
        let prompt = format!("{}% ", app_name);
        loop {
            let readline = rl.readline(prompt.as_str());
            match readline {
                Ok(line) => {
                    let params: Vec<&str> = line.trim().split(' ').map(str::trim).collect();
                    let cmd_name = params[0];
                    match cmd_name {
                        "quit" => {
                            let global_opt = Arc::try_unwrap(global_opt)
                                .ok()
                                .expect("unwrap opt must success when quit.");
                            let state = Arc::try_unwrap(state)
                                .ok()
                                .expect("unwrap state must success when quit.");
                            quit_action(app.clone(), global_opt, state);
                            break;
                        }
                        "help" => {
                            app.print_long_help().expect("print help should success.");
                        }
                        "version" => {
                            println!("{}", LONG_VERSION.as_str());
                        }
                        "console" => continue,
                        "" => continue,
                        cmd_name => {
                            let cmd = commands.get_mut(cmd_name);
                            match cmd {
                                Some(cmd) => {
                                    let app = cmd.get_app();
                                    match app.get_matches_from_safe_borrow(params) {
                                        Ok(arg_matches) => {
                                            let result = cmd.exec(
                                                state.clone(),
                                                global_opt.clone(),
                                                &arg_matches,
                                            );
                                            print_action_result(OutputFormat::TABLE, result)
                                                .expect("Print result should success.")
                                        }
                                        Err(e) => {
                                            println!("{}", e);
                                        }
                                    }
                                }
                                _ => {
                                    println!("Unknown command: {:?}", cmd_name);
                                    app.print_long_help().expect("print help should success.");
                                }
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
}
