//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::error;

enum Option {
    Single(String),
}

impl Option {
    fn is(&self, opt: &str) -> bool {
        match self {
            Self::Single(s) => return s == opt,
        }
    }
}

fn parse(optstring: &str) -> Vec<Option> {
    let mut ans = vec![];
    
    for c in optstring.chars() {
        let opt = format!("-{}", c);
        ans.push( Option::Single(opt) );
    }

    ans
}

pub fn getopts(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 3 {
        error::print("getopts: usage: getopts optstring name [arg ...]", core);
        return 2;
    }


    let targets = parse(&args[1]);
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

    if targets.iter().any(|t| t.is(&arg) ) {
        let result = core.db.set_param(&name, &arg);
        let _ = core.db.set_param("OPTIND", &(index+1).to_string());


        if let Err(e) = result {
            let msg = format!("getopts: {}", &e);
            error::print(&msg, core);
            return 1;
        }
    }

    0
}
