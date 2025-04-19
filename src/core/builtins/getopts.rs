//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{arg, error, ShellCore};

#[derive(Debug)]
enum Opt {
    Single(String),
    WithArg(String),
}

impl Opt {
    fn is_single(&self, opt: &str) -> bool {
        match self {
            Self::Single(s) => return s == opt,
            _ => false,
        }
    }

    fn is_witharg(&self, opt: &str) -> bool {
        match self {
            Self::WithArg(s) => return s == opt,
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
                Some(Opt::Single(opt)) => ans.push( Opt::WithArg(opt) ),
                _ => return (vec![], silence),
            }
        }else{
            let opt = format!("-{}", c);
            ans.push( Opt::Single(opt) );
        }
    }

    (ans, silence)
}

pub fn get_index(core: &mut ShellCore) -> (usize, usize) { //index, subindex
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

fn set_no_arg_option(name: &str, arg: &str, index: usize, subindex: usize,
                     subarg: bool, exp_args_len: usize, silence: bool,
                     core: &mut ShellCore, layer: Option<usize>) -> i32 {
    let result = core.db.set_param(&name, &arg[1..], layer);
    let _ = core.db.set_param("OPTARG", "", layer);

    if ! subarg || subindex + 1 == exp_args_len {
        let _ = core.db.set_param("OPTIND", &(index+1).to_string(), layer);
        let _ = core.db.set_param("OPTIND_SUB", "0", layer);
        let _ = core.db.set_param("OPTIND_PREV", &(index+1).to_string(), layer);
    }else{
        let _ = core.db.set_param("OPTIND", &index.to_string(), layer);
        let _ = core.db.set_param("OPTIND_SUB", &(subindex + 1).to_string(), layer);
        let _ = core.db.set_param("OPTIND_PREV", &index.to_string(), layer);
    }

    if let Err(e) = result {
        if ! silence {
            let msg = format!("getopts: {:?}", &e);
            error::print(&msg, core);
        }
        return 1;
    }
    return 0;
}

fn set_option_with_arg(name: &str, arg: &str, index: usize, optarg: &str,
                     silence: bool ,core: &mut ShellCore, layer: Option<usize>) -> i32 {
    let result = core.db.set_param(&name, &arg[1..], layer);

    let _ = core.db.set_param("OPTARG", &optarg, layer);
    let _ = core.db.set_param("OPTIND", &(index+2).to_string(), layer);
    let _ = core.db.set_param("OPTIND_PREV", &(index+2).to_string(), layer);
    let _ = core.db.set_param("OPTIND_SUB", "0", layer);

    if let Err(e) = result {
        let _ = core.db.set_param("OPTIND_PREV", &(index+10).to_string(), layer);
        if ! silence {
            let msg = format!("getopts: {:?}", &e);
            error::print(&msg, core);
        }
        return 1;
    }

    0
}

pub fn getopts(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let layer = core.db.get_layer_pos("OPTIND").unwrap_or(0);
    let layer_sub = core.db.get_layer_pos("OPTIND_SUB").unwrap_or(0);
    let layer_prev = core.db.get_layer_pos("OPTIND_PREV").unwrap_or(0);

    if layer_sub != layer {
        let _ = core.db.set_param("OPTIND_SUB", "0", Some(layer));
    }
    if layer_prev != layer {
        let _ = core.db.set_param("OPTIND_PREV", "0", Some(layer));
    }

    let _ = core.db.set_param("OPTARG", "", Some(layer));

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
    let _ = core.db.set_param(&name, "?", Some(layer));

    let args = match args.len() > 3 {
        true  => &args[2..],
        false => &core.db.position_parameters.last().unwrap().clone(),
    };

    if args.len() <= index {
        return 1;
    }

    let mut arg = args[index].clone();
    let exp_args = arg::dissolve_options(&vec![arg.clone()]);
    let subarg = exp_args.len() > 1;
    if subarg {
        if exp_args.len() <= subindex {
            return 1;
        }
        arg = exp_args[subindex].clone();
    }

    if ! arg.starts_with("-") {
        return 1;
    }

    if arg.starts_with("--") {
        let _ = core.db.set_param("OPTIND", &(index+1).to_string(), Some(layer));
        let _ = core.db.set_param("OPTIND_PREV", &(index+1).to_string(), Some(layer));
        return 1;
    }

    if targets.iter().any(|t| t.is_single(&arg) ) {
        return set_no_arg_option(&name, &arg, index, subindex, subarg, exp_args.len(), silence, core, Some(layer));
    }

    if targets.iter().any(|t| t.is_witharg(&arg) ) {
        let optarg = match args.len() > index+1 {
            true  => args[index+1].clone(),
            false => return 1,
        };

        return set_option_with_arg(&name, &arg, index, &optarg, silence, core, Some(layer));
    }

    1
}
