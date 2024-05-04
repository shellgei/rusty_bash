//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::word::Word;

pub fn compgen(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return 0;
    }

    match args[1].as_str() {
        "-W" => compgen_large_w(core, args),
        _ => {
            eprintln!("sush: compgen: {}: invalid option", &args[1]);
            return 2;
        },
    }
}

pub fn compgen_large_w(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        eprintln!("sush: compgen: -W: option requires an argument");
        return 2;
    }

    let mut ans: Vec<String> = vec![];
    let mut feeder = Feeder::new();
    feeder.add_line(args[2].to_string());
    while feeder.len() != 0 {
        match Word::parse(&mut feeder, core) {
            Some(mut w) => {
                w.unquote();
                ans.push(w.text)
            },
            _ => {
                let len = feeder.scanner_multiline_blank(core);
                feeder.consume(len);
            },
        }
    }

    if args.len() > 3 {
        ans.retain(|a| a.starts_with(&args[3]));
    }

    ans.iter().for_each(|a| println!("{}", a));
    0
}