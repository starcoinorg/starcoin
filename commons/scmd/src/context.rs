// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use crate::console::G_DEFAULT_CONSOLE_CONFIG;
use crate::console::{init_helper, CommandName, RLHelper};
use crate::error::CmdError;
use crate::{
    print_action_result, result_to_json, CommandAction, CommandExec, CustomCommand, HistoryOp,
    OutputFormat,
};
use anyhow::Result;
use clap::{error::ErrorKind, Parser};
use clap::{Arg, Command};
use rustyline::{error::ReadlineError, Config as ConsoleConfig, Editor};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

static G_OUTPUT_FORMAT_ARG: &str = "output-format";

pub struct CmdContext<State, GlobalOpt>
where
    State: 'static,
    GlobalOpt: Parser + 'static,
{
    app: Command,
    commands: HashMap<String, Box<dyn CommandExec<State, GlobalOpt>>>,
    default_action: Box<dyn FnOnce(Command, GlobalOpt, State)>,
    state_initializer: Box<dyn FnOnce(&GlobalOpt) -> Result<State>>,
    console_support: Option<(
        Box<dyn FnOnce(&Command, Arc<GlobalOpt>, Arc<State>) -> (ConsoleConfig, Option<PathBuf>)>,
        Box<dyn FnOnce(Command, GlobalOpt, State)>,
    )>,
}

impl<State, GlobalOpt> CmdContext<State, GlobalOpt>
where
    State: 'static,
    GlobalOpt: Parser + 'static,
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
        D: FnOnce(Command, GlobalOpt, State) + 'static,
    {
        let mut app = GlobalOpt::command();
        app = app
            .version(version)
            .long_version(long_version.unwrap_or(version))
            .arg(
                Arg::new(G_OUTPUT_FORMAT_ARG)
                    .short('o')
                    .help("set output-format, support [json|table]")
                    .num_args(1..)
                    .default_value("json"),
            );
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
            |_, _, _| -> (ConsoleConfig, Option<PathBuf>) { (*G_DEFAULT_CONSOLE_CONFIG, None) },
            |_, _, _| println!("Quit."),
        )
    }

    pub fn with_console_support<I, Q>(mut self, init_action: I, quit_action: Q) -> Self
    where
        I: FnOnce(&Command, Arc<GlobalOpt>, Arc<State>) -> (ConsoleConfig, Option<PathBuf>)
            + 'static,
        Q: FnOnce(Command, GlobalOpt, State) + 'static,
    {
        self.app = self.app.subcommand(
            Command::new("console").override_help("Start an interactive command console"),
        );
        self.console_support = Some((Box::new(init_action), Box::new(quit_action)));
        self
    }

    pub fn command<Opt, ReturnItem, Action, CMD>(mut self, command: CMD) -> Self
    where
        Opt: Parser + 'static,
        ReturnItem: serde::Serialize + 'static,
        Action: CommandAction<State = State, GlobalOpt = GlobalOpt, Opt = Opt, ReturnItem = ReturnItem>
            + 'static,
        CMD: Into<CustomCommand<State, GlobalOpt, Opt, ReturnItem, Action>> + 'static,
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

    fn app_help_message(app: &mut Command) -> String {
        let mut help_message = vec![];
        app.write_long_help(&mut help_message)
            .expect("format help message fail.");
        String::from_utf8(help_message).expect("help message should utf8")
    }

    /// Execute command by parse std::env::args_os() and print result.
    pub fn exec(self) -> Result<()> {
        let (output_format, result) = self.exec_inner(std::env::args_os())?;
        print_action_result(output_format, &result_to_json(&result))
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
        let matches = match app.try_get_matches_from_mut(iter) {
            Ok(matches) => matches,
            Err(err) => {
                return match err.kind() {
                    ErrorKind::DisplayVersion | ErrorKind::DisplayHelp => {
                        Ok((OutputFormat::TABLE, Ok(Value::String(err.to_string()))))
                    }
                    _ => Err(CmdError::ClapError(err).into()),
                };
            }
        };

        let output_format = matches
            .get_one::<String>(G_OUTPUT_FORMAT_ARG)
            .expect("output-format arg must exist")
            .parse()
            .expect("parse output-format must success.");

        let global_opt = GlobalOpt::from_arg_matches(&matches)?;
        let state = (self.state_initializer)(&global_opt)?;

        if let Some((cmd_name, arg_matches)) = matches.subcommand() {
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
                cmd_name => {
                    let cmd = self.commands.get_mut(cmd_name);
                    match (cmd, arg_matches) {
                        (Some(cmd), arg_matches) => {
                            let (_, value) =
                                cmd.exec(Arc::new(state), Arc::new(global_opt), arg_matches)?;
                            Ok(value)
                        }
                        _ => Err(CmdError::need_help(Self::app_help_message(&mut app)).into()),
                    }
                }
            };
            Ok((output_format, result))
        } else {
            (self.default_action)(app, global_opt, state);
            Ok((output_format, Ok(Value::Null)))
        }
    }

    fn console_inner(
        app: Command,
        global_opt: GlobalOpt,
        state: State,
        mut commands: HashMap<String, Box<dyn CommandExec<State, GlobalOpt>>>,
        init_action: Box<
            dyn FnOnce(&Command, Arc<GlobalOpt>, Arc<State>) -> (ConsoleConfig, Option<PathBuf>),
        >,
        quit_action: Box<dyn FnOnce(Command, GlobalOpt, State)>,
        mut output_format: OutputFormat,
    ) {
        //insert version, quit, history command
        let mut app = app
            .subcommand(
                Command::new("version")
                    .override_help("Print app version.")
                    .display_order(995),
            )
            .subcommand(
                Command::new("output")
                    .arg(
                        Arg::new("format")
                            .num_args(1..)
                            .value_parser(clap::builder::PossibleValuesParser::new(vec![
                                "json", "table",
                            ]))
                            .ignore_case(true)
                            .default_value("json")
                            .help("Output format should be json or table."),
                    )
                    .override_help("Set console output format.")
                    .display_order(996),
            )
            .subcommand(
                Command::new("history")
                    .arg(Arg::new("clear").short('c').help("Clear history."))
                    .override_help("Command to show or clear history")
                    .display_order(997),
            )
            .subcommand(
                Command::new("quit")
                    .aliases(&["exit", "q!"])
                    .override_help("Quit from console.")
                    .display_order(998),
            );

        let app_name = app.get_name().to_string();
        let global_opt = Arc::new(global_opt);
        let state = Arc::new(state);
        let (config, history_file) = init_action(&app, global_opt.clone(), state.clone());
        let mut rl = Editor::<RLHelper>::with_config(config);
        let cmd_sets = Self::get_command_names_recursively(&app, "".to_string(), 3)
            .iter()
            .map(|(a, b)| {
                CommandName::new(
                    a.to_string(),
                    b.replace(&app_name[..], "").trim().to_string(),
                )
            })
            .collect();

        rl.set_helper(Some(init_helper(cmd_sets)));
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
        let mut template_ctx = jpst::TemplateContext::new();
        let prompt = format!("{}% ", app_name);
        loop {
            let readline = rl.readline(prompt.as_str());
            match readline {
                Ok(line) => {
                    let line_after_eval = jpst::format_str!(line.as_str(), &template_ctx);
                    let params: Vec<&str> = line_after_eval
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
                            rl.add_history_entry(line.as_str());
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
                            rl.add_history_entry(line.as_str());
                            app.print_long_help().expect("print help should success.");
                        }
                        "version" => {
                            rl.add_history_entry(line.as_str());
                            let mut out = std::io::stdout();
                            let version = app.render_long_version();
                            out.write_all(version.as_bytes())
                                .expect("write version to stdout should be success");
                            // write a `\n` for flush stdout
                            out.write_all("\n".as_bytes())
                                .expect("write to stdout should success");
                        }
                        "output" => {
                            rl.add_history_entry(line.as_str());
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
                                    let app = cmd.get_command();
                                    match app.try_get_matches_from_mut(params) {
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
                                            let result_json = result_to_json(&result);

                                            match result {
                                                Ok(_) => {
                                                    if let Err(err) = print_action_result(
                                                        output_format,
                                                        &result_json,
                                                    ) {
                                                        println!("Print result error: {:?}", err);
                                                    }
                                                }
                                                Err(e) => println!("{}", e),
                                            }
                                            template_ctx.entry(cmd_name).append(result_json);
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
                    println!("Interrupted");
                    continue;
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
        app: Command,
        global_opt: Arc<GlobalOpt>,
        state: Arc<State>,
        quit_action: Box<dyn FnOnce(Command, GlobalOpt, State)>,
        mut rl: Editor<RLHelper>,
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

    fn get_command_names_recursively(
        app: &Command,
        prepositive: String,
        max_depth: u32,
    ) -> HashSet<(String, String)> {
        if max_depth == 0 {
            return HashSet::<(String, String)>::new();
        }
        let name = app.get_name();
        let mut pre = prepositive;
        if !pre.is_empty() {
            pre.push(' ');
        }
        pre.push_str(name);

        let mut set = HashSet::new();
        for sub_app in app.get_subcommands() {
            set.insert((sub_app.get_name().to_owned(), pre.clone()));
            let sub_set = Self::get_command_names_recursively(sub_app, pre.clone(), max_depth - 1);
            set.extend(sub_set);
        }
        set
    }
}
