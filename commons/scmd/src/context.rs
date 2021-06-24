// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::CmdError;
use crate::{print_action_result, Command, CommandAction, CommandExec, HistoryOp, OutputFormat};
use anyhow::Result;
use clap::{crate_authors, App, Arg, SubCommand};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use structopt::StructOpt;

pub use rustyline::{
    config::CompletionType, error::ReadlineError, ColorMode, Config as ConsoleConfig, EditMode,
    Editor,
};

pub static DEFAULT_CONSOLE_CONFIG: Lazy<ConsoleConfig> = Lazy::new(|| {
    ConsoleConfig::builder()
        .max_history_size(1000)
        .history_ignore_space(true)
        .history_ignore_dups(true)
        .completion_type(CompletionType::List)
        .auto_add_history(false)
        .edit_mode(EditMode::Emacs)
        .color_mode(ColorMode::Enabled)
        .build()
});

static OUTPUT_FORMAT_ARG: &str = "output-format";

pub struct CmdContext<State, GlobalOpt>
where
    State: 'static,
    GlobalOpt: StructOpt + 'static,
{
    app: App<'static, 'static>,
    commands: HashMap<String, Box<dyn CommandExec<State, GlobalOpt>>>,
    default_action: Box<dyn FnOnce(App, GlobalOpt, State)>,
    state_initializer: Box<dyn FnOnce(&GlobalOpt) -> Result<State>>,
    console_support: Option<(
        Box<dyn FnOnce(&App, Arc<GlobalOpt>, Arc<State>) -> (ConsoleConfig, Option<PathBuf>)>,
        Box<dyn FnOnce(App, GlobalOpt, State)>,
    )>,
}

impl<State, GlobalOpt> CmdContext<State, GlobalOpt>
where
    State: 'static,
    GlobalOpt: StructOpt + 'static,
{
    /// Init new CmdContext with State
    pub fn with_state(
        version: &'static str,
        long_version: Option<&'static str>,
        state: State,
    ) -> Self {
        Self::with_initializer(version, long_version, |_opts| Ok(state))
    }

    /// Init new CmdContext with state_initializer
    /// default action is print help.
    pub fn with_initializer<I>(
        version: &'static str,
        long_version: Option<&'static str>,
        state_initializer: I,
    ) -> Self
    where
        I: FnOnce(&GlobalOpt) -> Result<State> + 'static,
    {
        Self::with_default_action(
            version,
            long_version,
            state_initializer,
            |mut app, _opt, _state| {
                app.print_long_help().expect("print help should success.");
            },
        )
    }

    /// Init new CmdContext with state_initializer, and default_action
    /// default_action executed when no subcommand is provided.
    pub fn with_default_action<I, D>(
        version: &'static str,
        long_version: Option<&'static str>,
        state_initializer: I,
        default_action: D,
    ) -> Self
    where
        I: FnOnce(&GlobalOpt) -> Result<State> + 'static,
        D: FnOnce(App, GlobalOpt, State) + 'static,
    {
        let mut app = GlobalOpt::clap();
        app = app
            .version(version)
            .long_version(long_version.unwrap_or(version))
            .arg(
                Arg::with_name(OUTPUT_FORMAT_ARG)
                    .short("o")
                    .help("set output-format, support [json|table]")
                    .takes_value(true)
                    .default_value("json"),
            );
        app = Self::set_app_author(app);
        Self {
            app,
            commands: HashMap::new(),
            default_action: Box::new(default_action),
            state_initializer: Box::new(state_initializer),
            console_support: None,
        }
    }

    pub fn with_console_support_default(self) -> Self {
        self.with_console_support(
            |_, _, _| -> (ConsoleConfig, Option<PathBuf>) { (*DEFAULT_CONSOLE_CONFIG, None) },
            |_, _, _| println!("Quit."),
        )
    }

    pub fn with_console_support<I, Q>(mut self, init_action: I, quit_action: Q) -> Self
    where
        I: FnOnce(&App, Arc<GlobalOpt>, Arc<State>) -> (ConsoleConfig, Option<PathBuf>) + 'static,
        Q: FnOnce(App, GlobalOpt, State) + 'static,
    {
        self.app = self.app.subcommand(
            SubCommand::with_name("console").help("Start an interactive command console"),
        );
        self.console_support = Some((Box::new(init_action), Box::new(quit_action)));
        self
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
    pub fn exec(self) -> Result<()> {
        let (output_format, result) = self.exec_inner(&mut std::env::args_os())?;
        print_action_result(output_format, result, false)
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
        let matches = app
            .get_matches_from_safe_borrow(iter)
            .map_err(CmdError::ClapError)?;
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
                if let Some((init_action, quit_action)) = self.console_support {
                    let commands = self.commands;

                    Self::console_inner(
                        app,
                        global_opt,
                        state,
                        commands,
                        init_action,
                        quit_action,
                        output_format,
                    );
                    Ok(Value::Null)
                } else {
                    unreachable!("this should not happen, console cmd is check by clap.")
                }
            }
            "" => {
                default_action(app, global_opt, state);
                Ok(Value::Null)
            }
            cmd_name => {
                let cmd = self.commands.get_mut(cmd_name);
                match (cmd, arg_matches) {
                    (Some(cmd), Some(arg_matches)) => {
                        let (_, value) =
                            cmd.exec(Arc::new(state), Arc::new(global_opt), arg_matches)?;
                        Ok(value)
                    }
                    _ => Err(CmdError::need_help(Self::app_help_message(&mut app)).into()),
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
        init_action: Box<
            dyn FnOnce(&App, Arc<GlobalOpt>, Arc<State>) -> (ConsoleConfig, Option<PathBuf>),
        >,
        quit_action: Box<dyn FnOnce(App, GlobalOpt, State)>,
        mut output_format: OutputFormat,
    ) {
        //insert version, quit, history command
        let mut app = app
            .subcommand(
                SubCommand::with_name("version")
                    .help("Print app version.")
                    .display_order(995),
            )
            .subcommand(
                SubCommand::with_name("output")
                    .arg(Arg::from_usage("[format] 'Output format: JSON|TABLE'"))
                    .help("Set console output format.")
                    .display_order(996),
            )
            .subcommand(
                SubCommand::with_name("history")
                    .arg(Arg::from_usage("-c, --clear 'Clear console history.'"))
                    .help("Command to show or clear history")
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
        let (config, history_file) = init_action(&app, global_opt.clone(), state.clone());
        let mut rl = Editor::<()>::with_config(config);
        if let Some(history_file) = history_file.as_ref() {
            if !history_file.exists() {
                if let Err(e) = File::create(history_file.as_path()) {
                    println!("Create history file {:?} error: {:?}", history_file, e);
                }
            }
            if let Err(e) = rl.load_history(history_file.as_path()) {
                println!("Load history from file {:?} error: {:?}", history_file, e);
            }
        }
        let prompt = format!("{}% ", app_name);
        loop {
            let readline = rl.readline(prompt.as_str());
            match readline {
                Ok(line) => {
                    let params: Vec<&str> = line
                        .as_str()
                        .trim()
                        .split(' ')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .collect();
                    let cmd_name = if params.is_empty() { "" } else { params[0] };
                    match cmd_name {
                        "quit" | "exit" | "q!" => {
                            Self::do_quit(
                                app.clone(),
                                global_opt,
                                state,
                                quit_action,
                                rl,
                                history_file,
                            );
                            break;
                        }
                        "history" => {
                            if params.len() == 1 {
                                let history = rl.history();
                                for (idx, h_cmd) in history.iter().enumerate() {
                                    println!("{}:{}", idx, h_cmd);
                                }
                            } else if params.len() == 2
                                && (params[1] == "-c" || params[1] == "--clear")
                            {
                                let history = rl.history_mut();
                                let len = history.len();
                                history.clear();
                                if let Some(history_file) = history_file.as_ref() {
                                    if let Err(e) = rl.save_history(history_file.as_path()) {
                                        println!(
                                            "Save history to file {:?} error: {:?}",
                                            history_file, e
                                        );
                                    }
                                }
                                println!("Clear {} history command", len);
                            } else {
                                println!("Unexpect params: {:?} for history command.", params);
                            }
                        }
                        "help" => {
                            app.print_long_help().expect("print help should success.");
                        }
                        "version" => {
                            let mut out = std::io::stdout();
                            let _ = app
                                .write_long_version(&mut out)
                                .expect("write version to stdout should success");
                            // write a `\n` for flush stdout
                            out.write_all("\n".as_bytes())
                                .expect("write to stdout should success");
                        }
                        "output" => {
                            if params.len() == 1 {
                                println!("Current format: {}", output_format);
                            } else if params.len() == 2 {
                                output_format =
                                    OutputFormat::from_str(params[1]).unwrap_or_default();
                                println!("Set output format to: {}", output_format);
                            } else {
                                println!("Usage: output [format] 'Output format: JSON|TABLE'");
                            }
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
                                            let cmd_result = cmd.exec(
                                                state.clone(),
                                                global_opt.clone(),
                                                &arg_matches,
                                            );
                                            let (skip_history, result) = match cmd_result {
                                                Ok((history_op, value)) => (
                                                    matches!(history_op, HistoryOp::Skip),
                                                    Ok(value),
                                                ),
                                                Err(err) => (false, Err(err)),
                                            };
                                            if !skip_history {
                                                rl.add_history_entry(line.as_str());
                                            }
                                            if let Err(err) =
                                                print_action_result(output_format, result, true)
                                            {
                                                println!("Print result error: {:?}", err);
                                            }
                                        }
                                        Err(e) => {
                                            rl.add_history_entry(line.as_str());
                                            println!("{}", e);
                                        }
                                    }
                                }
                                _ => {
                                    rl.add_history_entry(line.as_str());
                                    println!("Unknown command: {:?}", cmd_name);
                                    app.print_long_help().expect("print help should success.");
                                }
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    Self::do_quit(
                        app.clone(),
                        global_opt,
                        state,
                        quit_action,
                        rl,
                        history_file,
                    );
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    Self::do_quit(
                        app.clone(),
                        global_opt,
                        state,
                        quit_action,
                        rl,
                        history_file,
                    );
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }

    fn do_quit(
        app: App,
        global_opt: Arc<GlobalOpt>,
        state: Arc<State>,
        quit_action: Box<dyn FnOnce(App, GlobalOpt, State)>,
        mut rl: Editor<()>,
        history_file: Option<PathBuf>,
    ) {
        let global_opt = Arc::try_unwrap(global_opt)
            .ok()
            .expect("unwrap opt must success when quit.");
        let state = Arc::try_unwrap(state)
            .ok()
            .expect("unwrap state must success when quit.");
        if let Some(history_file) = history_file.as_ref() {
            if let Err(e) = rl.save_history(history_file.as_path()) {
                println!("Save history to file {:?} error: {:?}", history_file, e);
            }
        }
        quit_action(app, global_opt, state);
    }
}
