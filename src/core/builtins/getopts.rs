//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{arg, error, ShellCore};

#[derive(Debug)]
enum Opt {
    Single(String),
    WithArg(String),
}

struct NoArgOpt<'a>{
    name: &'a str,
    arg: &'a str,
    index: usize,
    subindex: usize,
    subarg: bool,
    exp_args_len: usize,
    silence: bool,
    scope: Option<usize>,
}

impl Opt {
    fn is_single(&self, opt: &str) -> bool {
        match self {
            Self::Single(s) => s == opt,
            _ => false,
        }
    }

    fn is_witharg(&self, opt: &str) -> bool {
        match self {
            Self::WithArg(s) => s == opt,
            _ => false,
        }
    }
}

fn parse(optstring: &str) -> (Vec<Opt>, bool) {
    let mut optstring = optstring.to_string();
    let mut ans = vec![];
    let mut silence = false;

    if optstring.starts_with(":") {
        optstring.remove(0);
        silence = true;
    }

    for c in optstring.chars() {
        if c == ':' {
            match ans.pop() {
                Some(Opt::Single(opt)) => ans.push(Opt::WithArg(opt)),
                _ => return (vec![], silence),
            }
        } else {
            let opt = format!("-{c}");
            ans.push(Opt::Single(opt));
        }
    }

    (ans, silence)
}

pub fn get_index(core: &mut ShellCore) -> (usize, usize) {
    //index, subindex
    let mut index = 1;
    let mut subindex = 0;

    if let Ok(s) = core.db.get_param("OPTIND") {
        if let Ok(n) = s.parse::<usize>() {
            index = n;
        }

        if let Ok(p) = core.db.get_param("OPTIND_PREV") {
            if let Ok(prev) = p.parse::<usize>() {
                if index != prev {
                    let _ = core.db.set_param("OPTIND_SUB", "0", None);
                }
            }
        }
    }

    if let Ok(s) = core.db.get_param("OPTIND_SUB") {
        if let Ok(n) = s.parse::<usize>() {
            subindex = n;
        }
    }

    (index, subindex)
}

fn set_no_arg_option(no_arg_opt: &NoArgOpt, core: &mut ShellCore,) -> i32 {
    let result = core.db.set_param(no_arg_opt.name, &no_arg_opt.arg[1..], no_arg_opt.scope);
    core.db.set_param("OPTARG", "", no_arg_opt.scope).ok();

    if !no_arg_opt.subarg || no_arg_opt.subindex + 1 == no_arg_opt.exp_args_len {
        let _ = core.db.set_param("OPTIND", &(no_arg_opt.index + 1).to_string(), no_arg_opt.scope);
        let _ = core.db.set_param("OPTIND_SUB", "0", no_arg_opt.scope);
        let _ = core.db.set_param("OPTIND_PREV", &(no_arg_opt.index + 1).to_string(), no_arg_opt.scope);
    } else {
        let _ = core.db.set_param("OPTIND", &no_arg_opt.index.to_string(), no_arg_opt.scope);
        let _ = core.db.set_param("OPTIND_SUB", &(no_arg_opt.subindex + 1).to_string(), no_arg_opt.scope);
        let _ = core.db.set_param("OPTIND_PREV", &no_arg_opt.index.to_string(), no_arg_opt.scope);
    }

    if let Err(e) = result {
        if !no_arg_opt.silence {
            let msg = format!("getopts: {:?}", &e);
            error::print(&msg, core);
        }
        return 1;
    }
    0
}

fn set_option_with_arg(
    name: &str,
    arg: &str,
    index: usize,
    optarg: &str,
    silence: bool,
    core: &mut ShellCore,
    scope: Option<usize>,
) -> i32 {
    let result = core.db.set_param(name, &arg[1..], scope);

    let _ = core.db.set_param("OPTARG", optarg, scope);
    let _ = core.db.set_param("OPTIND", &(index + 2).to_string(), scope);
    let _ = core
        .db
        .set_param("OPTIND_PREV", &(index + 2).to_string(), scope);
    let _ = core.db.set_param("OPTIND_SUB", "0", scope);

    if let Err(e) = result {
        let _ = core
            .db
            .set_param("OPTIND_PREV", &(index + 10).to_string(), scope);
        if !silence {
            let msg = format!("getopts: {:?}", &e);
            error::print(&msg, core);
        }
        return 1;
    }

    0
}

pub fn getopts(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    let scope = core.db.get_scope_pos("OPTIND").unwrap_or(0);
    let scope_sub = core.db.get_scope_pos("OPTIND_SUB").unwrap_or(0);
    let scope_prev = core.db.get_scope_pos("OPTIND_PREV").unwrap_or(0);

    if scope_sub != scope {
        let _ = core.db.set_param("OPTIND_SUB", "0", Some(scope));
    }
    if scope_prev != scope {
        let _ = core.db.set_param("OPTIND_PREV", "0", Some(scope));
    }

    let _ = core.db.set_param("OPTARG", "", Some(scope));

    if args.len() < 3 {
        error::print("getopts: usage: getopts optstring name [arg ...]", core);
        return 2;
    }

    let (targets, silence) = parse(&args[1]);
    if targets.is_empty() {
        return 1;
    }

    let (index, subindex) = get_index(core);
    let name = args[2].clone();
    let _ = core.db.set_param(&name, "?", Some(scope));

    let args = match args.len() > 3 {
        true => &args[2..],
        false => &core.db.position_parameters.last().unwrap().clone(),
    };

    if args.len() <= index {
        return 1;
    }

    let mut arg = args[index].clone();
    let exp_args = arg::dissolve_options(&[arg.clone()]);
    let subarg = exp_args.len() > 1;
    if subarg {
        if exp_args.len() <= subindex {
            return 1;
        }
        arg = exp_args[subindex].clone();
    }

    if !arg.starts_with("-") {
        return 1;
    }

    if arg.starts_with("--") {
        let _ = core
            .db
            .set_param("OPTIND", &(index + 1).to_string(), Some(scope));
        let _ = core
            .db
            .set_param("OPTIND_PREV", &(index + 1).to_string(), Some(scope));
        return 1;
    }

    if targets.iter().any(|t| t.is_single(&arg)) {

        let no_arg_opt = NoArgOpt {
            name: &name,
            arg: &arg,
            index,
            subindex,
            subarg,
            exp_args_len: exp_args.len(),
            silence,
            scope: Some(scope),
        };
        return set_no_arg_option(&no_arg_opt, core);
    }

    if targets.iter().any(|t| t.is_witharg(&arg)) {
        let optarg = match args.len() > index + 1 {
            true => args[index + 1].clone(),
            false => return 1,
        };

        return set_option_with_arg(&name, &arg, index, &optarg, silence, core, Some(scope));
    }

    1
}
