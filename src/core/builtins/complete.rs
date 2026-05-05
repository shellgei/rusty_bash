//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2026 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::CompletionSpec;
use crate::{ShellCore, builtins};

#[derive(Default)]
struct CompleteParse {
    print: bool,
    remove: bool,
    default_scope: bool,
    empty_scope: bool,
    initial_scope: bool,
    names: Vec<String>,
    spec: CompletionSpec,
}

enum SpecialScope {
    Default,
    Empty,
    Initial,
}

const SHORT_ACTIONS: &[(char, &str)] = &[
    ('a', "alias"),
    ('b', "builtin"),
    ('c', "command"),
    ('d', "directory"),
    ('e', "export"),
    ('f', "file"),
    ('g', "group"),
    ('j', "job"),
    ('k', "keyword"),
    ('s', "service"),
    ('u', "user"),
    ('v', "variable"),
];

fn special_scope(parsed: &CompleteParse) -> Option<SpecialScope> {
    if parsed.default_scope {
        Some(SpecialScope::Default)
    } else if parsed.empty_scope {
        Some(SpecialScope::Empty)
    } else if parsed.initial_scope {
        Some(SpecialScope::Initial)
    } else {
        None
    }
}

fn action_to_reduce_symbol(arg: &str) -> Option<char> {
    SHORT_ACTIONS
        .iter()
        .find_map(|(flag, action)| (*action == arg).then_some(*flag))
}

fn opt_to_action(arg: &str) -> Option<&'static str> {
    let flag = arg.strip_prefix('-')?.chars().next()?;
    SHORT_ACTIONS
        .iter()
        .find_map(|(known, action)| (*known == flag).then_some(*action))
}

fn quote_if_needed(value: &str) -> String {
    if value.chars().any(char::is_whitespace) {
        format!("'{value}'")
    } else {
        value.to_string()
    }
}

fn print_each_complete(name: &str, spec: &CompletionSpec) -> i32 {
    print!("complete ");
    for option in &spec.options {
        print!("-o {option} ");
    }
    for action in &spec.actions {
        if let Some(symbol) = action_to_reduce_symbol(action) {
            print!("-{symbol} ");
        } else {
            print!("-A {action} ");
        }
    }
    if let Some(glob_pattern) = &spec.glob_pattern {
        print!("-G {} ", quote_if_needed(glob_pattern));
    }
    if let Some(wordlist) = &spec.wordlist {
        print!("-W {} ", quote_if_needed(wordlist));
    }
    if let Some(function) = &spec.function {
        print!("-F {function} ");
    }
    if let Some(command) = &spec.command {
        print!("-C {} ", quote_if_needed(command));
    }
    if let Some(filter_pattern) = &spec.filter_pattern {
        print!("-X {} ", quote_if_needed(filter_pattern));
    }
    if let Some(prefix) = &spec.prefix {
        print!("-P {} ", quote_if_needed(prefix));
    }
    if let Some(suffix) = &spec.suffix {
        print!("-S {} ", quote_if_needed(suffix));
    }
    println!("{name}");
    0
}

fn print_special_complete(flag: &str, spec: &Option<CompletionSpec>) -> i32 {
    if let Some(spec) = spec {
        print_each_complete(flag, spec);
    }
    0
}

fn print_complete_all(core: &mut ShellCore) -> i32 {
    print_special_complete("-D", &core.completion.default_spec);
    print_special_complete("-E", &core.completion.empty_spec);
    print_special_complete("-I", &core.completion.initial_spec);
    for (name, spec) in &core.completion.entries {
        print_each_complete(name, spec);
    }
    0
}

fn print_complete(parsed: &CompleteParse, core: &mut ShellCore) -> i32 {
    if let Some(scope) = special_scope(parsed) {
        match scope {
            SpecialScope::Default => print_special_complete("-D", &core.completion.default_spec),
            SpecialScope::Empty => print_special_complete("-E", &core.completion.empty_spec),
            SpecialScope::Initial => print_special_complete("-I", &core.completion.initial_spec),
        };
        return 0;
    }
    let mut err = false;
    for name in &parsed.names {
        if let Some(spec) = core.completion.entries.get(name) {
            print_each_complete(name, spec);
        } else {
            let err_str = format!("{name}: no completion specification");
            err = 0 != builtins::error_(1, "complete", &err_str, core);
        };
    }
    if err { 1 } else { 0 }
}

fn remove_complete(parsed: &CompleteParse, core: &mut ShellCore) -> i32 {
    if let Some(scope) = special_scope(parsed) {
        match scope {
            SpecialScope::Default => core.completion.default_spec = None,
            SpecialScope::Empty => core.completion.empty_spec = None,
            SpecialScope::Initial => core.completion.initial_spec = None,
        };
        return 0;
    }
    if parsed.names.is_empty() {
        core.completion.entries.clear();
        return 0;
    }
    for name in &parsed.names {
        if core.completion.entries.remove(name).is_none() {
            let err_str = format!("{name}: no completion specification");
            return builtins::error_(1, "complete", &err_str, core);
        }
    }
    0
}

fn install_complete(parsed: CompleteParse, core: &mut ShellCore) -> i32 {
    if let Some(scope) = special_scope(&parsed) {
        match scope {
            SpecialScope::Default => core.completion.default_spec = Some(parsed.spec),
            SpecialScope::Empty => core.completion.empty_spec = Some(parsed.spec),
            SpecialScope::Initial => core.completion.initial_spec = Some(parsed.spec),
        };
        return 0;
    }
    if parsed.names.is_empty() {
        return builtins::error_(2, "complete", "usage: complete: missing command name", core);
    }
    for name in parsed.names {
        core.completion.entries.insert(name, parsed.spec.clone());
    }
    0
}

fn missing_arg(core: &mut ShellCore, command: &str, opt: &str) -> Result<(), i32> {
    Err(builtins::error_(
        2,
        command,
        &format!("{opt}: option requires an argument"),
        core,
    ))
}

fn set_spec_arg(spec: &mut CompletionSpec, opt: &str, value: String) {
    match opt {
        "-o" => spec.options.push(value),
        "-A" => spec.actions.push(value),
        "-G" => spec.glob_pattern = Some(value),
        "-W" => spec.wordlist = Some(value),
        "-F" => spec.function = Some(value),
        "-C" => spec.command = Some(value),
        "-X" => spec.filter_pattern = Some(value),
        "-P" => spec.prefix = Some(value),
        "-S" => spec.suffix = Some(value),
        _ => unreachable!(),
    }
}

fn apply_complete_flag(parsed: &mut CompleteParse, flag: char) -> bool {
    if let Some(action) = SHORT_ACTIONS
        .iter()
        .find_map(|(known, action)| (*known == flag).then_some(*action))
    {
        parsed.spec.actions.push(action.to_string());
        return true;
    }
    match flag {
        'p' => parsed.print = true,
        'r' => parsed.remove = true,
        'D' => parsed.default_scope = true,
        'E' => parsed.empty_scope = true,
        'I' => parsed.initial_scope = true,
        _ => return false,
    }
    true
}

fn parse_complete(core: &mut ShellCore, args: &[String]) -> Result<CompleteParse, i32> {
    let mut parsed = CompleteParse::default();
    let command = args
        .first()
        .cloned()
        .unwrap_or_else(|| "complete".to_string());
    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--" => {
                parsed.names.extend(args[idx + 1..].iter().cloned());
                break;
            }
            "-p" => {
                parsed.print = true;
                idx += 1;
            }
            "-r" => {
                parsed.remove = true;
                idx += 1;
            }
            "-D" => {
                parsed.default_scope = true;
                idx += 1;
            }
            "-E" => {
                parsed.empty_scope = true;
                idx += 1;
            }
            "-I" => {
                parsed.initial_scope = true;
                idx += 1;
            }
            opt @ ("-o" | "-A" | "-G" | "-W" | "-F" | "-C" | "-X" | "-P" | "-S") => {
                if idx + 1 >= args.len() {
                    missing_arg(core, &command, opt)?;
                }
                set_spec_arg(&mut parsed.spec, opt, args[idx + 1].clone());
                idx += 2;
            }
            opt if opt.starts_with('-') && opt.len() > 2 => {
                for flag in opt[1..].chars() {
                    if !apply_complete_flag(&mut parsed, flag) {
                        let msg = format!("-{flag}: invalid option");
                        return Err(builtins::error_(2, &command, &msg, core));
                    }
                }
                idx += 1;
            }
            opt if opt.starts_with('-') => {
                if let Some(action) = opt_to_action(opt) {
                    parsed.spec.actions.push(action.to_string());
                    idx += 1;
                } else if let Some(flag) = opt.strip_prefix('-').and_then(|s| s.chars().next()) {
                    if !apply_complete_flag(&mut parsed, flag) {
                        let msg = format!("{opt}: invalid option");
                        return Err(builtins::error_(2, &command, &msg, core));
                    }
                    idx += 1;
                } else {
                    let msg = format!("{opt}: invalid option");
                    return Err(builtins::error_(2, &command, &msg, core));
                }
            }
            _ => {
                parsed.names.extend(args[idx..].iter().cloned());
                break;
            }
        }
    }
    parsed.spec.options.sort();
    parsed.spec.options.dedup();
    Ok(parsed)
}

pub fn complete(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() <= 1 {
        return print_complete_all(core);
    }
    let parsed = match parse_complete(core, args) {
        Ok(parsed) => parsed,
        Err(status) => return status,
    };
    if parsed.print {
        if !parsed.default_scope
            && !parsed.empty_scope
            && !parsed.initial_scope
            && parsed.names.is_empty()
        {
            return print_complete_all(core);
        }
        return print_complete(&parsed, core);
    }
    if parsed.remove {
        return remove_complete(&parsed, core);
    }
    install_complete(parsed, core)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_print_accepts_double_dash_before_names() {
        let mut core = ShellCore::new();
        core.completion.entries.insert(
            "make".to_string(),
            CompletionSpec {
                function: Some("_make".to_string()),
                ..Default::default()
            },
        );

        let args = ["complete", "-p", "--", "make"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert_eq!(complete(&mut core, &args), 0);
    }

    #[test]
    fn complete_invocation_replaces_existing_spec() {
        let mut core = ShellCore::new();
        let first = ["complete", "-F", "_cd", "cd"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let second = ["complete", "-d", "cd"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();

        assert_eq!(complete(&mut core, &first), 0);
        assert_eq!(complete(&mut core, &second), 0);

        let spec = core.completion.entries.get("cd").unwrap();
        assert_eq!(spec.function, None);
        assert_eq!(spec.actions, vec!["directory"]);
    }

    #[test]
    fn complete_keeps_combined_options_in_single_spec() {
        let mut core = ShellCore::new();
        let args = [
            "complete",
            "-d",
            "-W",
            "word",
            "-F",
            "_fn",
            "-o",
            "filenames",
            "x",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

        assert_eq!(complete(&mut core, &args), 0);

        let spec = core.completion.entries.get("x").unwrap();
        assert_eq!(spec.actions, vec!["directory"]);
        assert_eq!(spec.wordlist.as_deref(), Some("word"));
        assert_eq!(spec.function.as_deref(), Some("_fn"));
        assert!(spec.options.iter().any(|option| option == "filenames"));
    }

    #[test]
    fn complete_preserves_option_arguments_that_start_with_dash() {
        let mut core = ShellCore::new();
        let args = ["complete", "-W", "-x --long", "x"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();

        assert_eq!(complete(&mut core, &args), 0);

        let spec = core.completion.entries.get("x").unwrap();
        assert_eq!(spec.wordlist.as_deref(), Some("-x --long"));
    }

    #[test]
    fn complete_special_scope_uses_gnu_precedence() {
        let mut core = ShellCore::new();
        let args = ["complete", "-E", "-I", "-D", "-F", "_default", "ignored"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();

        assert_eq!(complete(&mut core, &args), 0);

        assert_eq!(
            core.completion
                .default_spec
                .as_ref()
                .unwrap()
                .function
                .as_deref(),
            Some("_default")
        );
        assert!(core.completion.empty_spec.is_none());
        assert!(core.completion.initial_spec.is_none());
        assert!(!core.completion.entries.contains_key("ignored"));
    }
}
