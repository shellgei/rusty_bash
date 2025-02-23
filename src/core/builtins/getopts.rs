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

pub fn getopts(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let _ = core.db.set_param("OPTARG", "", None);

    if args.len() < 3 {
        error::print("getopts: usage: getopts optstring name [arg ...]", core);
        return 2;
    }

    let (targets, silence) = parse(&args[1]);
    if targets.is_empty() {
        return 1;
    }

    let name = args[2].clone();
    let (index, subindex) = get_index(core);

    let args = match args.len() > 3 {
        true  => &args[2..],
        false => &core.db.position_parameters.last().unwrap(),
    };

    if args.len() <= index {
        return 1;
    }

    let mut subarg = false;
    let mut arg = args[index].clone();
    let exp_args = arg::dissolve_options(&vec![arg.clone()]);
    if exp_args.len() > 1 {
        if exp_args.len() > subindex {
            subarg = true;
            arg = exp_args[subindex].clone();
        }else{
            let _ = core.db.set_param(&name, "?", None);
            return 1;
        }
    }

    if ! arg.starts_with("-") {
        let _ = core.db.set_param(&name, "?", None);
        //let _ = core.db.set_param("OPTARG", "", None);
        return 1;
    }

    if arg.starts_with("--") {
        let _ = core.db.set_param(&name, "?", None);
        let _ = core.db.set_param("OPTARG", "?", None);
        let _ = core.db.set_param("OPTIND", &(index+1).to_string(), None);
        let _ = core.db.set_param("OPTIND_PREV", &(index+1).to_string(), None);
        return 1;
    }

    if targets.iter().any(|t| t.is_single(&arg) ) {
        let result = core.db.set_param(&name, &arg[1..], None);
        let _ = core.db.set_param("OPTARG", "", None);

        if ! subarg || subindex + 1 == exp_args.len() {
            let _ = core.db.set_param("OPTIND", &(index+1).to_string(), None);
            let _ = core.db.set_param("OPTIND_SUB", "0", None);
            let _ = core.db.set_param("OPTIND_PREV", &(index+1).to_string(), None);
        }else{
            let _ = core.db.set_param("OPTIND", &index.to_string(), None);
            let _ = core.db.set_param("OPTIND_SUB", &(subindex + 1).to_string(), None);
            let _ = core.db.set_param("OPTIND_PREV", &index.to_string(), None);
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

    let optarg = match args.len() > index+1 {
        true  => args[index+1].clone(),
        false => return 1,
    };

    if targets.iter().any(|t| t.is_witharg(&arg) ) {
        let result = core.db.set_param(&name, &arg[1..], None);

        let _ = core.db.set_param("OPTARG", &optarg, None);
        let _ = core.db.set_param("OPTIND", &(index+2).to_string(), None);
        let _ = core.db.set_param("OPTIND_PREV", &(index+2).to_string(), None);
        let _ = core.db.set_param("OPTIND_SUB", "0", None);

        if let Err(e) = result {
            let msg = format!("getopts: {:?}", &e);
            let _ = core.db.set_param("OPTIND_PREV", &(index+10).to_string(), None);
            error::print(&msg, core);
            return 1;
        }
    }

    0
}
