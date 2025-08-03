//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn alias(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() == 1 {
        for (k, v) in &core.aliases {
            println!("alias {k}='{v}'");
        }
        return 0;
    }

    if args.len() == 2 && args[1].contains("=") {
        let kv: Vec<String> = args[1].split("=").map(|t| t.to_string()).collect();
        core.aliases.insert(kv[0].clone(), kv[1..].join("="));
    }

    0
}

pub fn unalias(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() <= 1 {
        println!("unalias: usage: unalias [-a] name [name ...]");
    }

    if args.contains(&"-a".to_string()) {
        core.aliases.clear();
        return 0;
    }

    args[1..].iter().for_each(|e| {
        core.aliases.remove_entry(e);
    });

    0
}
