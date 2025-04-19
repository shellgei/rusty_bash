//SPDX-FileCopyrightText: 2025 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use rustyline::{Context, Helper};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter, CmdKind};
use rustyline::hint::Hinter;
use rustyline::validate::{Validator, MatchingBracketValidator, ValidationContext, ValidationResult};
use rustyline::validate::ValidationResult::Valid;
use std::borrow::Cow::{self, Borrowed, Owned};
use crate::feeder::terminal::completion;

pub struct SushHelper {
    pub completer: FilenameCompleter,
    pub highlighter: MatchingBracketHighlighter,
    pub validator: MatchingBracketValidator,
    pub colored_prompt: String,
}

impl SushHelper {
    pub fn new() -> Self {
        SushHelper { 
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            colored_prompt: String::new(),
        }
    }

    pub fn set_prompt(&mut self, p: String) {
        self.colored_prompt = p;
    }
}

impl Helper for SushHelper {}

impl Completer for SushHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        let text = &line[..pos];
        let tokens: Vec<&str> = text.split_whitespace().collect();
        if tokens.is_empty() || (tokens.len() == 1 && !tokens[0].contains('/')) {
            let prefix = tokens.get(0).copied().unwrap_or("");
            let comps = completion::get_commands(prefix);
            let start = text.find(prefix).unwrap_or(0);
            Ok((start, comps))
        } else {
            self.completer.complete(line, pos, ctx)
        }
    }
}

impl Hinter for SushHelper {
    type Hint = String;
    fn hint(&self, _: &str, _: usize, _: &Context<'_>) -> Option<String> { None }
}

impl Highlighter for SushHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, default: bool) -> Cow<'b, str> {
        if default { Borrowed(&self.colored_prompt) } else { Borrowed(prompt) }
    }
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> { self.highlighter.highlight(line, pos) }
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> { Owned(format!("\x1b[1m{}\x1b[m", hint)) }
    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

impl Validator for SushHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let res = self.validator.validate(ctx)?;
        if let ValidationResult::Incomplete = res { return Ok(ValidationResult::Incomplete); }
        // quotes check
        let inp = ctx.input();
        if inp.chars().filter(|&c| c == '\'').count() % 2 == 1 || inp.chars().filter(|&c| c == '"').count() % 2 == 1 {
            return Ok(ValidationResult::Incomplete);
        }
        Ok(Valid(None))
    }
}
