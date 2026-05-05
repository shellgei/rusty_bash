//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-FileCopyrightText: 2026 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod completion;
mod history;
mod prompt;

use crate::ShellCore;
use crate::error::input::InputError;
use history::{history_for_readline, remove_current_history_entry, update_current_history_entry};
use prompt::make_prompt_string;
use std::sync::atomic::Ordering::Relaxed;
use sushline::readline::{
    CompletionRequest, CompletionResponse, Editor, HistoryExpansionPolicy, Hooks, Prompt,
    ReadlineResult, Terminal, expand_history,
};

struct SushHooks<'a> {
    core: &'a mut ShellCore,
}

impl Hooks for SushHooks<'_> {
    fn check_signals(&self) -> Option<i32> {
        if self.core.sigint.load(Relaxed) {
            return Some(libc::SIGINT);
        }
        self.core
            .trapped
            .iter()
            .any(|(flag, _)| flag.load(Relaxed))
            .then_some(libc::SIGINT)
    }

    fn version(&self) -> Option<String> {
        Some(format!(
            "Sushi shell (a.k.a. Sush), version {}",
            env!("CARGO_PKG_VERSION")
        ))
    }

    fn expand_history(
        &mut self,
        context: sushline::readline::HistoryExpansionContext<'_>,
    ) -> Option<Result<Vec<u8>, String>> {
        let policy = HistoryExpansionPolicy {
            quotes_inhibit_expansion: true,
            ..HistoryExpansionPolicy::default()
        };
        Some(
            expand_history(
                context.line,
                context.history,
                context.histchars,
                &policy,
                |_| false,
            )
            .map_err(|err| err.message()),
        )
    }

    fn complete(&mut self, request: CompletionRequest) -> Option<CompletionResponse> {
        completion::programmable_completion(self.core, &request)
    }

    fn default_complete(&mut self, request: &CompletionRequest) -> Option<CompletionResponse> {
        completion::default_complete(self.core, request)
    }

    fn command_names(&self) -> Vec<String> {
        completion::command_names(self.core)
    }

    fn variable_names(&mut self) -> Vec<String> {
        completion::variable_names(self.core)
    }

    fn tokenize(&self, line: &[u8]) -> Option<Vec<Vec<u8>>> {
        Some(completion::tokenize(line))
    }

    fn completion_word_breaks(&self) -> Option<Vec<u8>> {
        Some(completion::completion_word_breaks())
    }
}

pub fn read_line(core: &mut ShellCore, prompt: &str) -> Result<String, InputError> {
    let extend_history_entry = prompt == "PS2" && !core.history.is_empty();
    let prompt = make_prompt_string(core, prompt);
    let history = history_for_readline(core);
    if !extend_history_entry {
        core.history.insert(0, String::new());
    }

    let mut line = core.readline.take().unwrap_or_else(|| {
        Editor::new(
            sushline::readline::Config::default(),
            Terminal::new(),
            sushline::readline::History::new(),
        )
    });
    *line.history_mut() = history;

    let result = {
        let mut hooks = SushHooks { core };
        line.read_line(Prompt::new(prompt), &mut hooks)
    };
    core.readline = Some(line);

    match result {
        Ok(ReadlineResult::Line(bytes)) => match readline_bytes_to_feeder_line(bytes) {
            Ok((input, history)) => {
                update_current_history_entry(core, extend_history_entry, &history);
                Ok(input)
            }
            Err(e) => {
                remove_current_history_entry(core);
                Err(e)
            }
        },
        Ok(ReadlineResult::Interrupted) => {
            core.sigint.store(true, Relaxed);
            remove_current_history_entry(core);
            Err(InputError::Interrupt)
        }
        Ok(ReadlineResult::Eof) => {
            remove_current_history_entry(core);
            Err(InputError::Eof)
        }
        Err(_) => {
            remove_current_history_entry(core);
            Err(InputError::Eof)
        }
    }
}

fn readline_bytes_to_feeder_line(bytes: Vec<u8>) -> Result<(String, String), InputError> {
    let mut input = String::from_utf8(bytes).map_err(|_| InputError::NotUtf8)?;
    let history = input.trim_end().to_string();
    input.push('\n');
    Ok((input, history))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_readline_line_keeps_feeder_trailing_newline() {
        let (input, history) =
            readline_bytes_to_feeder_line(b"echo \"alpha".to_vec()).expect("valid utf-8");
        assert_eq!(input, "echo \"alpha\n");
        assert_eq!(history, "echo \"alpha");
    }
}
