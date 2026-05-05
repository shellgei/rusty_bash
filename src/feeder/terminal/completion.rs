//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-FileCopyrightText: 2026 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::builtins::compgen;
use crate::core::completion::CompletionSpec;
use crate::elements::command;
use crate::elements::command::simple::SimpleCommand;
use crate::elements::io::pipe::Pipe;
use crate::elements::word::path_expansion;
use crate::utils::glob;
use crate::{Feeder, ShellCore, proc_ctrl, utils};
use nix::unistd;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::fd::RawFd;
use std::path::Path;
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

fn exact_completion_spec_for(core: &mut ShellCore, command: &str) -> Option<CompletionSpec> {
    core.completion.entries.get(command).cloned()
}

fn command_completion_spec_for(core: &mut ShellCore, command: &str) -> Option<CompletionSpec> {
    exact_completion_spec_for(core, command).or_else(|| {
        let name = command.rsplit('/').next()?;
        (name != command && !name.is_empty())
            .then(|| exact_completion_spec_for(core, name))
            .flatten()
    })
}

fn alias_completion_spec_for(core: &mut ShellCore, command: &str) -> Option<CompletionSpec> {
    if !core.db.has_array_value("BASH_ALIASES", command) {
        return None;
    }
    let alias = core.db.get_elem("BASH_ALIASES", command).ok()?;
    let command = utils::split_words(&alias).into_iter().next()?;
    command_completion_spec_for(core, &command)
}

fn run_complete_function(
    core: &mut ShellCore,
    spec: &CompletionSpec,
    command: &str,
    words: &[String],
    cword: usize,
) -> Option<i32> {
    let function = spec.function.as_ref()?;

    let target = words.get(cword).cloned().unwrap_or_default();
    let previous = cword
        .checked_sub(1)
        .and_then(|idx| words.get(idx))
        .cloned()
        .unwrap_or_default();
    let command = format!(
        "{} {} {} {}",
        function,
        quote_shell_word(command),
        quote_shell_word(&target),
        quote_shell_word(&previous)
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

fn apply_completion_affixes(mut candidates: Vec<String>, spec: &CompletionSpec) -> Vec<String> {
    if let Some(prefix) = &spec.prefix {
        candidates = candidates
            .into_iter()
            .map(|candidate| format!("{prefix}{candidate}"))
            .collect();
    }
    if let Some(suffix) = &spec.suffix {
        candidates = candidates
            .into_iter()
            .map(|candidate| format!("{candidate}{suffix}"))
            .collect();
    }
    candidates
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
    spec: &CompletionSpec,
    target: &str,
) -> Vec<String> {
    let mut candidates = Vec::new();
    for action in &spec.actions {
        candidates.extend(action_candidates(core, action, target));
    }
    candidates
}

fn response_from_candidates(
    candidates: Vec<String>,
    spec: Option<&CompletionSpec>,
    filenames: bool,
) -> CompletionResponse {
    let mut options = CompletionOptions {
        filenames,
        ..Default::default()
    };
    if let Some(spec) = spec {
        options.nospace = spec.has_option("nospace");
        options.noquote = spec.has_option("noquote");
        options.nosort = spec.has_option("nosort");
        options.fullquote = spec.has_option("fullquote");
        options.bashdefault = spec.has_option("bashdefault");
        options.default = spec.has_option("default");
        options.dirnames = spec.has_option("dirnames");
        options.plusdirs = spec.has_option("plusdirs");
        options.filenames |= spec.filenames();
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

fn completion_spec_for(
    core: &mut ShellCore,
    request: &CompletionRequest,
    words: &[String],
    cword: usize,
) -> Option<CompletionSpec> {
    if words.is_empty() {
        return core
            .completion
            .empty_spec
            .clone()
            .or_else(|| core.completion.default_spec.clone());
    }
    if cword == 0 && is_shell_command_position(request) {
        return core
            .completion
            .initial_spec
            .clone()
            .or_else(|| core.completion.default_spec.clone());
    }
    let command = words.first()?;
    command_completion_spec_for(core, command)
        .or_else(|| core.completion.default_spec.clone())
        .or_else(|| alias_completion_spec_for(core, command))
}

fn wordlist_candidates(core: &mut ShellCore, spec: &CompletionSpec, target: &str) -> Vec<String> {
    let Some(wordlist) = spec.wordlist.as_ref() else {
        return Vec::new();
    };
    let args = vec![
        String::new(),
        "-W".to_string(),
        wordlist.clone(),
        target.to_string(),
    ];
    compgen::compgen_large_w(core, &args)
}

fn glob_candidates(core: &mut ShellCore, spec: &CompletionSpec) -> Vec<String> {
    spec.glob_pattern
        .as_ref()
        .map(|pattern| path_expansion::expand(pattern, &core.shopts))
        .unwrap_or_default()
}

fn apply_completion_filter(core: &ShellCore, candidates: &mut Vec<String>, spec: &CompletionSpec) {
    let Some(pattern) = spec.filter_pattern.as_ref() else {
        return;
    };
    let extglob = core.shopts.query("extglob");
    candidates.retain(|candidate| !glob::parse_and_compare(candidate, pattern, extglob));
}

fn quote_shell_word(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn read_completion_command_output(core: &mut ShellCore, fd: RawFd) -> Vec<String> {
    let file = core.fds.get_file(fd);
    let reader = BufReader::new(file);
    reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.is_empty())
        .collect()
}

fn run_complete_command(
    core: &mut ShellCore,
    spec: &CompletionSpec,
    command: &str,
    words: &[String],
    cword: usize,
) -> Vec<String> {
    let Some(program) = spec.command.as_ref() else {
        return Vec::new();
    };
    let target = words.get(cword).cloned().unwrap_or_default();
    let previous = cword
        .checked_sub(1)
        .and_then(|idx| words.get(idx))
        .cloned()
        .unwrap_or_default();
    let command_line = format!(
        "{} {} {} {}",
        program,
        quote_shell_word(command),
        quote_shell_word(&target),
        quote_shell_word(&previous)
    );
    let mut feeder = Feeder::new(&command_line);
    let Ok(Some(mut command)) = command::parse(&mut feeder, core) else {
        return Vec::new();
    };
    let mut pipe = Pipe::new("|".to_string());
    pipe.set(-1, unistd::getpgrp(), core);
    let Ok(pid) = command.exec(core, &mut pipe) else {
        return Vec::new();
    };
    let candidates = read_completion_command_output(core, pipe.recv);
    if let Some(pid) = pid {
        proc_ctrl::wait_pipeline(core, vec![Some(pid)], false);
    }
    candidates
}

pub(super) fn programmable_completion(
    core: &mut ShellCore,
    request: &CompletionRequest,
) -> Option<CompletionResponse> {
    let (_, words, cword) = set_completion_variables(core, request);
    let command = words.first().cloned().unwrap_or_default();
    let target = words.get(cword).cloned().unwrap_or_default();
    let mut spec = completion_spec_for(core, request, &words, cword)?;
    let _ = core
        .db
        .init_array("COMPREPLY", Some(Vec::new()), None, false);

    let mut function_candidates = Vec::new();
    for _ in 0..4 {
        core.completion.current = spec.clone();
        let status = run_complete_function(core, &spec, &command, &words, cword);
        if status == Some(124)
            && let Some(next_spec) = command_completion_spec_for(core, &command)
        {
            spec = next_spec;
            let _ = core
                .db
                .init_array("COMPREPLY", Some(Vec::new()), None, false);
            continue;
        }
        if status.is_some() {
            function_candidates = core.db.get_vec("COMPREPLY", true).unwrap_or_default();
        }
        break;
    }

    let effective_spec = core.completion.current.clone();
    let mut candidates = Vec::new();
    candidates.extend(entry_action_candidates(core, &effective_spec, &target));
    candidates.extend(glob_candidates(core, &effective_spec));
    candidates.extend(wordlist_candidates(core, &effective_spec, &target));
    candidates.extend(function_candidates);
    candidates.extend(run_complete_command(
        core,
        &effective_spec,
        &command,
        &words,
        cword,
    ));
    apply_completion_filter(core, &mut candidates, &effective_spec);
    candidates = apply_completion_affixes(candidates, &effective_spec);
    let response = response_from_candidates(
        candidates,
        Some(&effective_spec),
        effective_spec.filenames(),
    );
    if response.candidates.is_empty() && !effective_spec.allows_default_completion() {
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
    let candidates = command_candidates(core, &target);
    (!candidates.is_empty()).then(|| response_from_candidates(candidates, None, false))
}

fn command_candidates(core: &mut ShellCore, target: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    candidates.extend(
        core.db
            .get_indexes_all("BASH_ALIASES")
            .iter()
            .filter(|name| name.starts_with(target))
            .cloned(),
    );
    candidates.extend(
        core.builtins
            .keys()
            .filter(|name| name.starts_with(target))
            .cloned(),
    );
    candidates.extend(
        core.db
            .functions
            .keys()
            .filter(|name| name.starts_with(target))
            .cloned(),
    );
    candidates.extend(compgen::compgen_k(&[
        String::new(),
        String::new(),
        target.to_string(),
    ]));
    candidates.extend(path_command_candidates(core, target));
    candidates.sort();
    candidates.dedup();
    candidates
}

fn path_command_candidates(core: &mut ShellCore, target: &str) -> Vec<String> {
    core.db
        .get_param("PATH")
        .unwrap_or_default()
        .split(':')
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flat_map(|entries| entries.flatten())
        .filter_map(|entry| {
            let path = entry.path();
            if !is_executable_command(&path) {
                return None;
            }
            let name = entry.file_name().into_string().ok()?;
            name.starts_with(target).then_some(name)
        })
        .collect()
}

#[cfg(unix)]
fn is_executable_command(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_command(path: &Path) -> bool {
    path.is_file()
}

fn static_command_names(core: &ShellCore) -> Vec<String> {
    let mut candidates = Vec::new();
    candidates.extend(core.builtins.keys().cloned());
    candidates.extend(core.db.functions.keys().cloned());
    candidates.extend(compgen::compgen_k(&[]));
    candidates.sort();
    candidates.dedup();
    candidates
}

pub(super) fn command_names(core: &ShellCore) -> Vec<String> {
    static_command_names(core)
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

    static PROCESS_ENV_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();

    fn process_env_guard() -> std::sync::MutexGuard<'static, ()> {
        PROCESS_ENV_LOCK
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

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

    fn has_candidate(response: &CompletionResponse, replacement: &[u8]) -> bool {
        response
            .candidates
            .iter()
            .any(|candidate| candidate.replacement == replacement)
    }

    #[test]
    fn shell_default_completion_only_runs_at_command_position() {
        assert!(is_shell_command_position(&request(b"ec", 0, b"ec")));
        assert!(is_shell_command_position(&request(b"true && ec", 8, b"ec")));
        assert!(!is_shell_command_position(&request(b"echo src", 5, b"src")));
    }

    #[test]
    fn programmable_completion_honors_compopt_changes() {
        let _guard = process_env_guard();
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        run_script(
            &mut core,
            "__comp() { COMPREPLY=(foo); compopt -o nospace; }",
        );
        core.completion.entries.insert(
            "x".to_string(),
            CompletionSpec {
                function: Some("__comp".to_string()),
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
        let _guard = process_env_guard();
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        run_script(
            &mut core,
            "__load() { complete -W 'loaded' foo; return 124; }",
        );
        core.completion.default_spec = Some(CompletionSpec {
            function: Some("__load".to_string()),
            ..Default::default()
        });

        let response = programmable_completion(&mut core, &request(b"foo ", 4, b"")).unwrap();

        assert_eq!(response.candidates.len(), 1);
        assert_eq!(response.candidates[0].replacement, b"loaded");
    }

    #[test]
    fn programmable_completion_does_not_retry_loader_without_status_124() {
        let _guard = process_env_guard();
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        run_script(
            &mut core,
            "__load() { complete -W 'loaded' foo; return 1; }",
        );
        core.completion.default_spec = Some(CompletionSpec {
            function: Some("__load".to_string()),
            ..Default::default()
        });

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
            CompletionSpec {
                actions: vec!["builtin".to_string()],
                ..Default::default()
            },
        );
        core.completion.entries.insert(
            "shopt".to_string(),
            CompletionSpec {
                actions: vec!["shopt".to_string()],
                ..Default::default()
            },
        );

        let builtin_response =
            programmable_completion(&mut core, &request(b"builtin ec", 8, b"ec")).unwrap();
        assert!(has_candidate(&builtin_response, b"echo"));

        let shopt_response =
            programmable_completion(&mut core, &request(b"shopt prog", 6, b"prog")).unwrap();
        assert!(has_candidate(&shopt_response, b"progcomp"));
    }

    #[test]
    fn programmable_completion_combines_spec_generators() {
        let _guard = process_env_guard();
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        run_script(
            &mut core,
            "__comp() { COMPREPLY=(from_func); }; complete -W 'from_wordlist' -F __comp x",
        );

        let response = programmable_completion(&mut core, &request(b"x ", 2, b"")).unwrap();

        assert!(has_candidate(&response, b"from_wordlist"));
        assert!(has_candidate(&response, b"from_func"));
    }

    #[test]
    fn programmable_completion_uses_pathname_basename_spec() {
        let _guard = process_env_guard();
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        run_script(&mut core, "complete -W 'from_basename' foo");

        let response = programmable_completion(&mut core, &request(b"./foo ", 6, b"")).unwrap();

        assert!(has_candidate(&response, b"from_basename"));
    }

    #[test]
    fn programmable_completion_uses_alias_target_spec() {
        let _guard = process_env_guard();
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        run_script(
            &mut core,
            "alias ll='foo --flag'; complete -W 'from_alias' foo",
        );

        let response = programmable_completion(&mut core, &request(b"ll ", 3, b"")).unwrap();

        assert!(has_candidate(&response, b"from_alias"));
    }

    #[test]
    fn file_actions_set_filename_option() {
        for action in ["file", "directory"] {
            let spec = CompletionSpec {
                actions: vec![action.to_string()],
                ..Default::default()
            };
            let response = response_from_candidates(vec!["src".to_string()], Some(&spec), false);

            assert!(response.options.filenames);
            assert!(has_candidate(&response, b"src"));
        }
    }

    #[test]
    fn default_command_completion_includes_keywords() {
        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();

        let response = default_complete(&mut core, &request(b"wh", 0, b"wh")).unwrap();

        assert!(has_candidate(&response, b"while"));
    }

    #[test]
    fn command_completion_uses_command_namespace() {
        let _guard = process_env_guard();
        let current_dir = std::env::current_dir().unwrap();
        let temp_dir = std::env::temp_dir().join(format!("sush-completion-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir(&temp_dir).unwrap();
        let bin_dir = temp_dir.join("bin");
        std::fs::create_dir(&bin_dir).unwrap();
        std::fs::create_dir(temp_dir.join("src")).unwrap();
        let runme = temp_dir.join("runme");
        std::fs::write(&runme, "#!/bin/sh\n").unwrap();
        let external = bin_dir.join("external-run");
        std::fs::write(&external, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&runme).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&runme, permissions).unwrap();
            let mut permissions = std::fs::metadata(&external).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&external, permissions).unwrap();
        }
        std::env::set_current_dir(&temp_dir).unwrap();

        let mut core = ShellCore::new();
        core.db.position_parameters[0] = vec!["sush".to_string()];
        core.configure().unwrap();
        let _ = core
            .db
            .set_param("PATH", &bin_dir.display().to_string(), None);
        run_script(
            &mut core,
            "alias ealias='echo alias'; __local_function() { :; }",
        );

        let builtin_response = default_complete(&mut core, &request(b"ec", 0, b"ec")).unwrap();
        let alias_response = default_complete(&mut core, &request(b"eal", 0, b"eal")).unwrap();
        let function_response =
            default_complete(&mut core, &request(b"__loc", 0, b"__loc")).unwrap();
        let path_response = default_complete(&mut core, &request(b"ext", 0, b"ext")).unwrap();
        let local_directory_response = default_complete(&mut core, &request(b"sr", 0, b"sr"));
        let local_executable_response = default_complete(&mut core, &request(b"ru", 0, b"ru"));

        std::env::set_current_dir(current_dir).unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
        assert!(has_candidate(&builtin_response, b"echo"));
        assert!(has_candidate(&alias_response, b"ealias"));
        assert!(has_candidate(&function_response, b"__local_function"));
        assert!(has_candidate(&path_response, b"external-run"));
        assert!(local_directory_response.is_none());
        assert!(local_executable_response.is_none());
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
