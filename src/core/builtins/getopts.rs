//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error;

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
        }

        let opt = format!("-{}", c);
        ans.push( Opt::Single(opt) );
    }

    (ans, silence)
}

pub fn getopts(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 3 {
        error::print("getopts: usage: getopts optstring name [arg ...]", core);
        return 2;
    }

    let (targets, silence) = parse(&args[1]);
    if targets.is_empty() {
        return 1;
    }

    let name = args[2].clone();
    let mut index = 1;

    if let Ok(s) = core.db.get_param("OPTIND") {
        if let Ok(n) = s.parse::<usize>() {
            index = n;
        }
    }

    let args = match args.len() > 3 {
        true  => &args[2..],
        false => &core.db.position_parameters.last().unwrap(),
    };

    if args.len() <= index {
        return 1;
    }

    let arg = args[index].clone();

    if ! arg.starts_with("-") {
        let _ = core.db.set_param(&name, "?", None);
        let _ = core.db.set_param("OPTARG", "?", None);
        return 1;
    }

    if arg.starts_with("--") {
        let _ = core.db.set_param(&name, "?", None);
        let _ = core.db.set_param("OPTARG", "?", None);
        let _ = core.db.set_param("OPTIND", &(index+1).to_string(), None);
        return 1;
    }

    if targets.iter().any(|t| t.is_single(&arg) ) {
        let result = core.db.set_param(&name, &arg[1..], None);
        let _ = core.db.set_param("OPTIND", &(index+1).to_string(), None);
        let _ = core.db.set_param("OPTARG", "", None);

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

        if let Err(e) = result {
            let msg = format!("getopts: {:?}", &e);
            error::print(&msg, core);
            return 1;
        }
    }

    0
}
