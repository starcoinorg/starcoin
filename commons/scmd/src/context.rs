// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::CmdError;
use crate::{print_action_result, Command, CommandAction, CommandExec, OutputFormat};
use anyhow::Result;
use clap::{crate_authors, crate_version, App, Arg, ArgMatches, SubCommand};
use git_version::git_version;
use lazy_static::lazy_static;
use rustyline::{config::CompletionType, error::ReadlineError, Config, Editor};
use std::collections::HashMap;
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
    default_action: Box<dyn Fn(App, GlobalOpt, State)>,
    state_initializer: Box<dyn Fn(&GlobalOpt) -> Result<State>>,
    console_start_action: Box<dyn Fn(&App, Arc<GlobalOpt>, Arc<State>)>,
    console_quit_action: Box<dyn Fn(App, GlobalOpt, State)>,
}

impl<State, GlobalOpt> CmdContext<State, GlobalOpt>
where
    GlobalOpt: StructOpt,
{
    pub fn new<I>(state_initializer: I) -> Self
    where
        I: Fn(&GlobalOpt) -> Result<State> + 'static,
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
        I: Fn(&GlobalOpt) -> Result<State> + 'static,
        D: Fn(App, GlobalOpt, State) + 'static,
        S: Fn(&App, Arc<GlobalOpt>, Arc<State>) + 'static,
        Q: Fn(App, GlobalOpt, State) + 'static,
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
        let mut help_message = vec![];
        self.app
            .write_long_help(&mut help_message)
            .expect("format help message fail.");
        String::from_utf8(help_message).expect("help message should utf8")
    }

    pub fn exec(mut self) {
        match self.exec_inner() {
            Err(e) => println!("{}", e.to_string()),
            Ok(_) => {}
        }
    }

    fn exec_inner(&mut self) -> Result<()> {
        let matches = self
            .app
            .get_matches_from_safe_borrow(&mut std::env::args_os())?;
        let output_format = matches
            .value_of(OUTPUT_FORMAT_ARG)
            .expect("output-format arg must exist")
            .parse()
            .expect("parse output-format must success.");

        let (global_opt, state) = self.init_global_opt(&matches)?;
        let (cmd_name, arg_matches) = matches.subcommand();
        match cmd_name {
            "console" => {
                self.console_inner(global_opt, state);
            }
            "" => {
                self.default_action.as_ref()(self.app.clone(), global_opt, state);
            }
            cmd_name => {
                let cmd = self.commands.get_mut(cmd_name);
                match (cmd, arg_matches) {
                    (Some(cmd), Some(arg_matches)) => {
                        let value = cmd.exec(Arc::new(state), Arc::new(global_opt), arg_matches)?;
                        print_action_result(value, output_format)?;
                    }
                    _ => {
                        return Err(CmdError::NeedHelp {
                            help: self.help_message(),
                        }
                        .into());
                    }
                };
            }
        }

        Ok(())
    }

    fn console_inner(&mut self, global_opt: GlobalOpt, state: State) {
        //TODO support use custom config
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .auto_add_history(true)
            .build();
        //insert quit command
        let mut app = self
            .app
            .clone()
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
        self.console_start_action.as_ref()(&app, global_opt.clone(), state.clone());
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
                            self.console_quit_action.as_ref()(app.clone(), global_opt, state);
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
                            let cmd = self.commands.get_mut(cmd_name);
                            match cmd {
                                Some(cmd) => {
                                    let app = cmd.get_app();
                                    match app.get_matches_from_safe_borrow(params) {
                                        Ok(arg_matches) => {
                                            match cmd.exec(
                                                state.clone(),
                                                global_opt.clone(),
                                                &arg_matches,
                                            ) {
                                                Ok(v) => {
                                                    print_action_result(v, OutputFormat::TABLE)
                                                        .expect("Print result should success.")
                                                }
                                                Err(e) => println!("{}", e),
                                            };
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

    fn init_global_opt(&self, matches: &ArgMatches) -> Result<(GlobalOpt, State)> {
        let global_opt = GlobalOpt::from_clap(&matches);
        let state = self.state_initializer.as_ref()(&global_opt)?;
        Ok((global_opt, state))
    }

    pub fn console(mut self) {
        let matches = self
            .app
            .get_matches_from_safe_borrow(&mut std::env::args_os())
            .unwrap_or_else(|e| panic!("{}", e));
        let (global_opt, state) = self
            .init_global_opt(&matches)
            .unwrap_or_else(|e| panic!("{}", e));
        self.console_inner(global_opt, state);
    }
}
