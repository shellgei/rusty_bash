//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2026 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::CompletionSpec;
use crate::utils::arg;

fn compopt_set(info: &mut CompletionSpec, plus: &[String], minus: &[String]) -> i32 {
    for opt in minus {
        if !info.options.contains(opt) {
            info.options.push(opt.to_string());
        }
    }

    for opt in plus {
        info.options.retain(|e| e != opt);
    }
    info.options.sort();

    0
}

fn compopt_print(core: &mut ShellCore, args: &[String]) -> i32 {
    let optlist = [
        "bashdefault",
        "default",
        "dirnames",
        "filenames",
        "noquote",
        "nosort",
        "nospace",
        "plusdirs",
    ];
    let optlist: Vec<String> = optlist.iter().map(|s| s.to_string()).collect();

    let com = args[1].clone();
    if core.completion.entries.contains_key(&com) {
        let info = &core.completion.entries.get_mut(&com).unwrap();

        print!("compopt ");
        for opt in &optlist {
            match info.options.contains(opt) {
                true => print!("-o {opt} "),
                false => print!("+o {opt} "),
            }
        }
        println!("{}", &com);
    } else {
        eprintln!("sush: compopt: {}: no completion specification", &args[1]);
        return 1;
    }

    0
}

pub fn compopt(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = args.to_owned();
    if args.len() < 2 {
        dbg!("{:?}", &core.completion.entries);
        return 1;
    }

    if !args[1].starts_with("-") && !args[1].starts_with("+") {
        return compopt_print(core, &args);
    }

    let mut minus = vec![];
    let mut plus = vec![];
    let mut default_scope = false;
    let mut empty_scope = false;
    let mut initial_scope = false;

    while args.len() > 1 {
        if args[1] == "-D" {
            default_scope = true;
            args.remove(1);
            continue;
        }
        if args[1] == "-E" {
            empty_scope = true;
            args.remove(1);
            continue;
        }
        if args[1] == "-I" {
            initial_scope = true;
            args.remove(1);
            continue;
        }

        if args[1] == "-o" {
            let opt = arg::consume_with_next_arg("-o", &mut args);
            if opt.is_none() {
                return 1;
            }

            minus.push(opt.unwrap());
            continue;
        }

        if args[1] == "+o" {
            let opt = arg::consume_with_next_arg("+o", &mut args);
            if opt.is_none() {
                return 1;
            }

            plus.push(opt.unwrap());
            continue;
        }

        break;
    }

    if default_scope {
        return core
            .completion
            .default_spec
            .as_mut()
            .map(|info| compopt_set(info, &plus, &minus))
            .unwrap_or(1);
    }
    if empty_scope {
        return core
            .completion
            .empty_spec
            .as_mut()
            .map(|info| compopt_set(info, &plus, &minus))
            .unwrap_or(1);
    }
    if initial_scope {
        return core
            .completion
            .initial_spec
            .as_mut()
            .map(|info| compopt_set(info, &plus, &minus))
            .unwrap_or(1);
    }

    let info = if args.len() == 1 {
        &mut core.completion.current
    } else if args.len() == 2 {
        match core.completion.entries.get_mut(&args[1]) {
            Some(i) => i,
            None => return 1,
        }
    } else {
        return 1;
    };
    compopt_set(info, &plus, &minus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compopt_special_scope_uses_gnu_precedence() {
        let mut core = ShellCore::new();
        core.completion.default_spec = Some(CompletionSpec::default());
        core.completion.empty_spec = Some(CompletionSpec::default());
        core.completion.initial_spec = Some(CompletionSpec::default());
        let args = [
            "compopt",
            "-E",
            "-o",
            "filenames",
            "-D",
            "-o",
            "nospace",
            "-I",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

        assert_eq!(compopt(&mut core, &args), 0);

        let default_spec = core.completion.default_spec.as_ref().unwrap();
        assert!(default_spec.has_option("filenames"));
        assert!(default_spec.has_option("nospace"));
        assert!(
            !core
                .completion
                .empty_spec
                .as_ref()
                .unwrap()
                .has_option("filenames")
        );
        assert!(
            !core
                .completion
                .initial_spec
                .as_ref()
                .unwrap()
                .has_option("nospace")
        );
    }
}
