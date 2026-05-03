//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-FileCopyrightText: 2026 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::builtins::compgen;
use crate::core::completion::CompletionEntry;
use crate::elements::command::simple::SimpleCommand;
use crate::elements::io::pipe::Pipe;
use crate::utils::arg;
use crate::{Feeder, ShellCore, utils};
use sushline::readline::{
    CompletionCandidate, CompletionOptions, CompletionRequest, CompletionResponse,
};

const BASH_COMP_WORDBREAKS: &[u8] = b" \t\n\"'@><=;|&(:";
const COMP_TYPE_COMPLETE: &str = "\t";
const COMP_KEY_TAB: &str = "9";
const SHELL_COMMAND_POSITION_PRECEDERS: &[u8] = b";|&({";
const BASH_SPECIAL_PARAMETER_NAMES: &[&str] = &["?", "$", "!", "#", "-", "_", "0"];

fn completion_words(line: &str, point: usize) -> (Vec<String>, usize) {
    let point = point.min(line.len());
    let all_words = utils::split_words(line);
    let left_words = utils::split_words(&line[..point]);
    let mut cword = left_words.len();
    if !line[..point].ends_with([' ', '\t']) {
        cword = cword.saturating_sub(1);
    }
    (all_words, cword)
}

fn completion_entry_for(core: &mut ShellCore, command: &str) -> Option<CompletionEntry> {
    exact_completion_entry_for(core, command).or_else(|| {
        (!core.completion.default_function.is_empty()).then(|| CompletionEntry {
            function: core.completion.default_function.clone(),
            ..Default::default()
        })
    })
}

fn exact_completion_entry_for(core: &mut ShellCore, command: &str) -> Option<CompletionEntry> {
    core.completion.entries.get(command).cloned()
}

fn run_complete_function(
    core: &mut ShellCore,
    entry: &CompletionEntry,
    command: &str,
    words: &[String],
    cword: usize,
) -> Option<i32> {
    if entry.function.is_empty() {
        return None;
    }

    let target = words.get(cword).cloned().unwrap_or_default();
    let previous = cword
        .checked_sub(1)
        .and_then(|idx| words.get(idx))
        .cloned()
        .unwrap_or_default();
    let command = format!(
        "{} \"{}\" \"{}\" \"{}\"",
        entry.function, command, target, previous
    );
    let mut feeder = Feeder::new(&command);
    if let Ok(Some(mut command)) = SimpleCommand::parse(&mut feeder, core) {
        let mut pipe = Pipe::new(String::new());
        if command.exec(core, &mut pipe).is_ok() {
            return Some(core.db.exit_status);
        }
    }
    None
}

fn apply_completion_affixes(mut candidates: Vec<String>, entry: &CompletionEntry) -> Vec<String> {
    if let Some(prefix) = entry.options.get("-P") {
        candidates = candidates
            .into_iter()
            .map(|candidate| format!("{prefix}{candidate}"))
            .collect();
    }
    if let Some(suffix) = entry.options.get("-S") {
        candidates = candidates
            .into_iter()
            .map(|candidate| format!("{candidate}{suffix}"))
            .collect();
    }
    candidates
}

fn entry_option(entry: &CompletionEntry, option: &str) -> bool {
    arg::has_option(option, &entry.o_options)
}

fn entry_filenames(entry: &CompletionEntry) -> bool {
    entry_option(entry, "filenames")
        || entry_option(entry, "fullquote")
        || entry
            .actions
            .iter()
            .any(|action| matches!(action.as_str(), "file" | "directory"))
}

fn entry_allows_fallback(entry: &CompletionEntry) -> bool {
    ["bashdefault", "default", "dirnames", "plusdirs"]
        .iter()
        .any(|option| entry_option(entry, option))
}

fn action_candidates(core: &mut ShellCore, action: &str, target: &str) -> Vec<String> {
    let args = vec![String::new(), String::new(), target.to_string()];
    match action {
        "alias" => compgen::compgen_a(core, &args),
        "builtin" => compgen::compgen_b(core, &args),
        "command" => compgen::compgen_c(core, &args),
        "directory" => compgen::compgen_f(core, &args, true),
        "export" => compgen::compgen_e(&args),
        "file" => compgen::compgen_f(core, &args, false),
        "function" => compgen::compgen_function(core, &args),
        "hostname" => compgen::compgen_hostname(core, &args),
        "job" => compgen::compgen_j(core, &args),
        "keyword" => compgen::compgen_k(&args),
        "setopt" => compgen::compgen_o(core, &args),
        "shopt" => compgen::compgen_shopt(core, &args),
        "stopped" => compgen::compgen_stopped(core, &args),
        "user" => compgen::compgen_u(core, &args),
        "variable" => compgen::compgen_v(core, &args),
        _ => Vec::new(),
    }
}

fn entry_action_candidates(
    core: &mut ShellCore,
    entry: &CompletionEntry,
    target: &str,
) -> Vec<String> {
    let mut candidates = Vec::new();
    for action in &entry.actions {
        candidates.extend(action_candidates(core, action, target));
    }
    candidates.sort();
    candidates.dedup();
    candidates
}

fn response_from_candidates(
    candidates: Vec<String>,
    entry: Option<&CompletionEntry>,
    filenames: bool,
) -> CompletionResponse {
    let mut options = CompletionOptions {
        filenames,
        ..Default::default()
    };
    if let Some(entry) = entry {
        options.nospace = entry_option(entry, "nospace");
        options.noquote = entry_option(entry, "noquote");
        options.nosort = entry_option(entry, "nosort");
        options.fullquote = entry_option(entry, "fullquote");
        options.bashdefault = entry_option(entry, "bashdefault");
        options.default = entry_option(entry, "default");
        options.dirnames = entry_option(entry, "dirnames");
        options.plusdirs = entry_option(entry, "plusdirs");
        options.filenames |= entry_filenames(entry);
    }
    CompletionResponse {
        candidates: candidates
            .into_iter()
            .filter(|candidate| !candidate.is_empty())
            .map(|candidate| CompletionCandidate {
                replacement: candidate.into_bytes(),
                display: None,
            })
            .collect(),
        options,
    }
}

fn is_shell_command_position(request: &CompletionRequest) -> bool {
    let start = request.context.word_start.min(request.context.line.len());
    let prefix = &request.context.line[..start];
    let Some(last) = prefix
        .iter()
        .rev()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
    else {
        return true;
    };
    SHELL_COMMAND_POSITION_PRECEDERS.contains(&last)
}

fn set_completion_variables(
    core: &mut ShellCore,
    request: &CompletionRequest,
) -> (String, Vec<String>, usize) {
    let line = String::from_utf8_lossy(&request.context.line).into_owned();
    let point = request.context.point.min(line.len());
    let (words, cword) = completion_words(&line, point);

    let _ = core.db.set_param("COMP_LINE", &line, None);
    let _ = core.db.set_param("COMP_POINT", &point.to_string(), None);
    let _ = core.db.set_param("COMP_TYPE", COMP_TYPE_COMPLETE, None);
    let _ = core.db.set_param("COMP_KEY", COMP_KEY_TAB, None);
    let _ = core
        .db
        .init_array("COMP_WORDS", Some(words.clone()), None, false);
    let _ = core.db.set_param("COMP_CWORD", &cword.to_string(), None);
    (line, words, cword)
}

pub(super) fn programmable_completion(
    core: &mut ShellCore,
    request: &CompletionRequest,
) -> Option<CompletionResponse> {
    let (_, words, cword) = set_completion_variables(core, request);
    let command = words.first().cloned().unwrap_or_default();
    let target = words.get(cword).cloned().unwrap_or_default();
    let mut entry = completion_entry_for(core, &command)?;
    let _ = core
        .db
        .init_array("COMPREPLY", Some(Vec::new()), None, false);

    let mut handled = false;
    for _ in 0..4 {
        core.completion.current = entry.clone();
        let status = run_complete_function(core, &entry, &command, &words, cword);
        if status == Some(124)
            && let Some(next_entry) = exact_completion_entry_for(core, &command)
        {
            entry = next_entry;
            let _ = core
                .db
                .init_array("COMPREPLY", Some(Vec::new()), None, false);
            continue;
        }
        handled = status.is_some();
        break;
    }

    if !handled {
        core.completion.current = entry.clone();
    }

    let mut candidates = if handled {
        core.db.get_vec("COMPREPLY", true).unwrap_or_default()
    } else if !entry.large_w_cands.is_empty() {
        utils::split_words(&entry.large_w_cands)
            .into_iter()
            .filter(|candidate| candidate.starts_with(&target))
            .collect()
    } else if !entry.actions.is_empty() {
        entry_action_candidates(core, &entry, &target)
    } else {
        Vec::new()
    };

    candidates = apply_completion_affixes(candidates, &core.completion.current);
    let response = response_from_candidates(
        candidates,
        Some(&core.completion.current),
        entry_filenames(&core.completion.current),
    );
    if response.candidates.is_empty() && !entry_allows_fallback(&entry) {
        return Some(response);
    }
    Some(response)
}

pub(super) fn default_complete(
    core: &mut ShellCore,
    request: &CompletionRequest,
) -> Option<CompletionResponse> {
    if !is_shell_command_position(request) {
        return None;
    }
    let target = String::from_utf8_lossy(&request.context.word).into_owned();
    let args = vec![String::new(), String::new(), target];
    Some(response_from_candidates(
        compgen::compgen_c(core, &args),
        None,
        false,
    ))
}

pub(super) fn command_names(core: &ShellCore) -> Vec<String> {
    let mut names: Vec<String> = core.builtins.keys().cloned().collect();
    names.extend(core.db.functions.keys().cloned());
    names.extend(compgen::compgen_k(&[]));
    names.sort();
    names.dedup();
    names
}

pub(super) fn variable_names(core: &mut ShellCore) -> Vec<String> {
    let mut names = core.db.get_param_keys();
    for name in BASH_SPECIAL_PARAMETER_NAMES {
        if !names.iter().any(|known| known == name) {
            names.push(name.to_string());
        }
    }
    for (name, _) in std::env::vars() {
        if !names.iter().any(|known| known == &name) {
            names.push(name);
        }
    }
    names.sort();
    names
}

pub(super) fn tokenize(line: &[u8]) -> Vec<Vec<u8>> {
    let line = String::from_utf8_lossy(line);
    utils::split_words(&line)
        .into_iter()
        .map(String::into_bytes)
        .collect()
}

pub(super) fn completion_word_breaks() -> Vec<u8> {
    BASH_COMP_WORDBREAKS.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::script::Script;
    use sushline::readline::{CompletionContext, CompletionType};

    fn request(line: &[u8], word_start: usize, word: &[u8]) -> CompletionRequest {
        CompletionRequest {
            context: CompletionContext {
                line: line.to_vec(),
                point: line.len(),
                word_start,
                word_end: line.len(),
                word: word.to_vec(),
                key: b"\t".to_vec(),
                completion_type: CompletionType::Complete,
            },
        }
    }

    fn run_script(core: &mut ShellCore, text: &str) {
        let mut feeder = Feeder::new(text);
        let mut script = Script::parse(&mut feeder, core, false).unwrap().unwrap();
        script.exec(core).unwrap();
    }

    #[test]
    fn shell_default_completion_only_runs_at_command_position() {
        assert!(is_shell_command_position(&request(b"ec", 0, b"ec")));
        assert!(is_shell_command_position(&request(b"true && ec", 8, b"ec")));
        assert!(!is_shell_command_position(&request(b"echo src", 5, b"src")));
    }

    #[test]
    fn programmable_completion_honors_compopt_changes() {
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        run_script(
            &mut core,
            "__comp() { COMPREPLY=(foo); compopt -o nospace; }",
        );
        core.completion.entries.insert(
            "x".to_string(),
            CompletionEntry {
                function: "__comp".to_string(),
                ..Default::default()
            },
        );

        let response = programmable_completion(&mut core, &request(b"x f", 2, b"f")).unwrap();

        assert_eq!(response.candidates.len(), 1);
        assert_eq!(response.candidates[0].replacement, b"foo");
        assert!(response.options.nospace);
    }

    #[test]
    fn programmable_completion_retries_after_loader_status_124() {
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        run_script(
            &mut core,
            "__load() { complete -W 'loaded' foo; return 124; }",
        );
        core.completion.default_function = "__load".to_string();

        let response = programmable_completion(&mut core, &request(b"foo ", 4, b"")).unwrap();

        assert_eq!(response.candidates.len(), 1);
        assert_eq!(response.candidates[0].replacement, b"loaded");
    }

    #[test]
    fn programmable_completion_does_not_retry_loader_without_status_124() {
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        run_script(
            &mut core,
            "__load() { complete -W 'loaded' foo; return 1; }",
        );
        core.completion.default_function = "__load".to_string();

        let response = programmable_completion(&mut core, &request(b"foo ", 4, b"")).unwrap();

        assert!(response.candidates.is_empty());
    }

    #[test]
    fn programmable_completion_supports_registered_builtin_actions() {
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        core.completion.entries.insert(
            "builtin".to_string(),
            CompletionEntry {
                actions: vec!["builtin".to_string()],
                ..Default::default()
            },
        );
        core.completion.entries.insert(
            "shopt".to_string(),
            CompletionEntry {
                actions: vec!["shopt".to_string()],
                ..Default::default()
            },
        );

        let builtin_response =
            programmable_completion(&mut core, &request(b"builtin ec", 8, b"ec")).unwrap();
        assert!(
            builtin_response
                .candidates
                .iter()
                .any(|candidate| candidate.replacement == b"echo")
        );

        let shopt_response =
            programmable_completion(&mut core, &request(b"shopt prog", 6, b"prog")).unwrap();
        assert!(
            shopt_response
                .candidates
                .iter()
                .any(|candidate| candidate.replacement == b"progcomp")
        );
    }

    #[test]
    fn default_command_completion_includes_keywords() {
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();

        let response = default_complete(&mut core, &request(b"wh", 0, b"wh")).unwrap();

        assert!(
            response
                .candidates
                .iter()
                .any(|candidate| candidate.replacement == b"while")
        );
    }

    #[test]
    fn command_names_include_keywords_for_readline_commands() {
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();

        let names = command_names(&core);

        assert!(names.iter().any(|name| name == "if"));
        assert!(names.iter().any(|name| name == "while"));
    }
}
