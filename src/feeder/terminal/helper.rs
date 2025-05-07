//SPDX-FileCopyrightText: 2025 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use rustyline::{Context, Helper};
use rustyline::completion::{Completer, FilenameCompleter, Pair, Candidate};
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
        let prefix = &line[..pos];
        // 最初のword
        let word_st = prefix.find(|c: char| !c.is_whitespace()).unwrap_or(pos);
        let after_st = &prefix[word_st..];
        let word_len = after_st.find(char::is_whitespace).unwrap_or(after_st.len());
        let word_end = word_st + word_len;
        let first_w = &line[word_st..word_end];

        if pos <= word_end {
            let cmd_pref = &prefix[word_st..pos];
            if !cmd_pref.contains('/') {
                // '/'なしならコマンド補完
                let comps = completion::get_commands(cmd_pref);
                Ok((word_st, comps))
            } else {
                // '/'ありならファイル名補完
                self.completer.complete(line, pos, ctx)
            }
        } else { // 2番目以降の引数はファイル名補完
            let (start, mut cands) = self.completer.complete(line, pos, ctx)?;
            
            // 'cd' コマンドの場合はディレクトリのみ
            if first_w == "cd" {
                cands.retain(|p| {
                    // line="cd ", start=3, p.replacement()="dir/" -> res_line="cd dir/"
                    let res_line = format!("{}{}", &line[0..start], p.replacement());
                    let cmd_len = first_w.len();

                    if res_line.len() > cmd_len && 
                       res_line.as_bytes().get(cmd_len) == Some(&b' ') {
                        // "cd dir/" -> " dir/" -> "dir/"
                        let arg_str = res_line[cmd_len..].trim_start();

                        if !arg_str.is_empty() {
                            let pth = std::path::Path::new(arg_str.trim_end_matches('/'));
                            return pth.is_dir();
                        }
                    }
                    false
                });
            }
            Ok((start, cands))
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
