// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow::{self, Borrowed, Owned};

use once_cell::sync::Lazy;
use rustyline::completion::{extract_word, Completer, FilenameCompleter, Pair};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::Context;
use rustyline_derive::Helper;
use std::collections::HashSet;

use rustyline::{
    config::CompletionType, error::ReadlineError, ColorMode, Config as ConsoleConfig, EditMode,
    OutputStreamType,
};

pub static G_DEFAULT_CONSOLE_CONFIG: Lazy<ConsoleConfig> = Lazy::new(|| {
    ConsoleConfig::builder()
        .max_history_size(1000)
        .history_ignore_space(true)
        .history_ignore_dups(true)
        .completion_type(CompletionType::List)
        .auto_add_history(false)
        .edit_mode(EditMode::Emacs)
        .color_mode(ColorMode::Enabled)
        .output_stream(OutputStreamType::Stdout)
        .build()
});

const DEFAULT_BREAK_CHARS: [u8; 3] = [b' ', b'\t', b'\n'];

#[derive(Hash, Debug, PartialEq, Eq)]
pub(crate) struct CommandName {
    cmd: String,
    pre_cmd: String,
}

impl CommandName {
    pub(crate) fn new(cmd: String, pre_cmd: String) -> Self {
        Self { cmd, pre_cmd }
    }
}
struct CommandCompleter {
    cmds: HashSet<CommandName>,
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

pub(crate) fn init_helper(cmds: HashSet<CommandName>) -> RLHelper {
    RLHelper {
        file_completer: FilenameCompleter::new(),
        cmd_completer: CommandCompleter { cmds },
        highlighter: MatchingBracketHighlighter::new(),
        validator: MatchingBracketValidator::new(),
    }
}
