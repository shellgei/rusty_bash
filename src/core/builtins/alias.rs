//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn alias(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() == 1 {
        for k in &core.db.get_indexes_all("BASH_ALIASES") {
            let v = core.db.get_elem("BASH_ALIASES", k).unwrap();
            println!("alias {k}='{v}'");
        }
        return 0;
    }

    if args.len() == 2 && args[1].contains("=") {
        let kv: Vec<String> = args[1].split('=').map(|t| t.to_string()).collect();
        //core.aliases.insert(kv[0].clone(), kv[1..].join("="));
        let _ = core
            .db
            .set_assoc_elem("BASH_ALIASES", &kv[0], &kv[1..].join("="), None);
    }

    0
}

pub fn unalias(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() <= 1 {
        println!("unalias: usage: unalias [-a] name [name ...]");
    }

    if args.iter().any(|s| s == "-a") {
        core.db.unset("BASH_ALIASES");
        let _ = core.db.set_assoc("BASH_ALIASES", None, true);
        return 0;
    }

    args[1..].iter().for_each(|e| {
        let _ = core.db.unset_array_elem("BASH_ALIASES", e);
    });

    0
}
