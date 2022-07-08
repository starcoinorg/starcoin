use std::borrow::Cow::{self, Borrowed, Owned};

// use rustyline::Result;
use rustyline::completion::{extract_word, Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::Context;
use rustyline_derive::Helper;
use std::collections::HashSet;

const DEFAULT_BREAK_CHARS: [u8; 3] = [b' ', b'\t', b'\n'];

#[derive(Hash, Debug, PartialEq, Eq)]
struct Command {
    cmd: String,
    pre_cmd: String,
}

impl Command {
    fn new(cmd: &str, pre_cmd: &str) -> Self {
        Self {
            cmd: cmd.into(),
            pre_cmd: pre_cmd.into(),
        }
    }
}
struct CommandCompleter {
    cmds: HashSet<Command>,
}

impl CommandCompleter {
    pub fn find_matches(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start, word) = extract_word(line, pos, None, &DEFAULT_BREAK_CHARS);
        let pre_cmd = line[..start].trim();

        let matches = self
            .cmds
            .iter()
            .filter_map(|hint| {
                if hint.cmd.starts_with(word) && pre_cmd == hint.pre_cmd {
                    let mut replacement = hint.cmd.clone();
                    replacement += " ";
                    Some(Pair {
                        display: hint.cmd.to_string(),
                        replacement: replacement.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok((start, matches))
    }
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.find_matches(line, pos)
    }
}
impl Hinter for CommandCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        None
    }
}

#[derive(Helper)]
pub(crate) struct RLHelper {
    file_completer: FilenameCompleter,
    cmd_completer: CommandCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
}

impl Completer for RLHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        match self.cmd_completer.complete(line, pos, ctx) {
            Ok((start, matches)) => {
                if matches.is_empty() {
                    self.file_completer.complete(line, pos, ctx)
                } else {
                    Ok((start, matches))
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl Hinter for RLHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for RLHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for RLHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

// Commands need to be auto-completed
// TODO: auto fetch commands from Clap
fn cmd_sets() -> HashSet<Command> {
    let mut set = HashSet::new();
    set.insert(Command::new("account", ""));
    set.insert(Command::new("state", ""));
    set.insert(Command::new("node", ""));
    set.insert(Command::new("chain", ""));
    set.insert(Command::new("txpool", ""));
    set.insert(Command::new("dev", ""));
    set.insert(Command::new("contract", ""));
    set.insert(Command::new("version", ""));
    set.insert(Command::new("output", ""));
    set.insert(Command::new("history", ""));
    set.insert(Command::new("quit", ""));
    set.insert(Command::new("console", ""));
    set.insert(Command::new("help", ""));

    // Subcommand of account
    set.insert(Command::new("create", "account"));
    set.insert(Command::new("show", "account"));
    set.insert(Command::new("transfer", "account"));
    set.insert(Command::new("accept-token", "account"));
    set.insert(Command::new("list", "account"));
    set.insert(Command::new("import-multisig", "account"));
    set.insert(Command::new("change-password", "account"));
    set.insert(Command::new("default", "account"));
    set.insert(Command::new("remove", "account"));
    set.insert(Command::new("lock", "account"));
    set.insert(Command::new("unlock", "account"));
    set.insert(Command::new("export", "account"));
    set.insert(Command::new("import", "account"));
    set.insert(Command::new("import-readonly", "account"));
    set.insert(Command::new("execute-function", "account"));
    set.insert(Command::new("execute-script", "account"));
    set.insert(Command::new("sign-multisig-txn", "account"));
    set.insert(Command::new("submit-txn", "account"));
    set.insert(Command::new("sign-message", "account"));
    set.insert(Command::new("verify-sign-message", "account"));
    set.insert(Command::new("derive-address", "account"));
    set.insert(Command::new("receipt-identifier", "account"));
    set.insert(Command::new("generate-keypair", "account"));
    set.insert(Command::new("rotate-authentication-key", "account"));
    set.insert(Command::new("nft", "account"));
    set.insert(Command::new("help", "account"));

    // Subcommad of state
    set.insert(Command::new("list", "state"));
    set.insert(Command::new("get", "state"));
    set.insert(Command::new("get-proof", "state"));
    set.insert(Command::new("get-root", "state"));
    set.insert(Command::new("help", "state"));

    // Subcommad of node
    set.insert(Command::new("info", "node"));
    set.insert(Command::new("peers", "node"));
    set.insert(Command::new("metrics", "node"));
    set.insert(Command::new("manager", "node"));
    set.insert(Command::new("service", "node"));
    set.insert(Command::new("sync", "node"));
    set.insert(Command::new("network", "node"));
    set.insert(Command::new("help", "node"));

    // Subcommad of chain
    set.insert(Command::new("info", "chain"));
    set.insert(Command::new("get-block", "chain"));
    set.insert(Command::new("list-block", "chain"));
    set.insert(Command::new("get-txn", "chain"));
    set.insert(Command::new("get-txn-infos", "chain"));
    set.insert(Command::new("get-txn-info", "chain"));
    set.insert(Command::new("get-events", "chain"));
    set.insert(Command::new("epoch-info", "chain"));
    set.insert(Command::new("get-txn-info-list", "chain"));
    set.insert(Command::new("get-txn-proof", "chain"));
    set.insert(Command::new("get-block-info", "chain"));
    set.insert(Command::new("help", "chain"));

    // Subcommad of txpool
    set.insert(Command::new("pending-txn", "txpool"));
    set.insert(Command::new("pending-txns", "txpool"));
    set.insert(Command::new("status", "txpool"));
    set.insert(Command::new("help", "txpool"));

    // Subcommad of dev
    set.insert(Command::new("get-coin", "dev"));
    set.insert(Command::new("move-explain", "dev"));
    set.insert(Command::new("compile", "dev"));
    set.insert(Command::new("deploy", "dev"));
    set.insert(Command::new("module-proposal", "dev"));
    set.insert(Command::new("module-plan", "dev"));
    set.insert(Command::new("module-queue", "dev"));
    set.insert(Command::new("module-exe", "dev"));
    set.insert(Command::new("vm-config-proposal", "dev"));
    set.insert(Command::new("package", "dev"));
    set.insert(Command::new("call", "dev"));
    set.insert(Command::new("resolve", "dev"));
    set.insert(Command::new("call-api", "dev"));
    set.insert(Command::new("subscribe", "dev"));
    set.insert(Command::new("log", "dev"));
    set.insert(Command::new("panic", "dev"));
    set.insert(Command::new("sleep", "dev"));
    set.insert(Command::new("gen-block", "dev"));
    set.insert(Command::new("help", "dev"));

    // Subcommad of contract
    set.insert(Command::new("get", "contract"));
    set.insert(Command::new("help", "contract"));
    // Subcommad of version
    // Subcommad of output
    // Subcommad of history
    // Subcommad of quit
    // Subcommad of console
    set
}

pub(crate) fn init_helper() -> RLHelper {
    RLHelper {
        file_completer: FilenameCompleter::new(),
        cmd_completer: CommandCompleter { cmds: cmd_sets() },
        highlighter: MatchingBracketHighlighter::new(),
        validator: MatchingBracketValidator::new(),
    }
}
