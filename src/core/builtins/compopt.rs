//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::CompletionEntry;
use crate::utils::arg;
use crate::ShellCore;

fn compopt_set(info: &mut CompletionEntry, plus: &Vec<String>, minus: &Vec<String>) -> i32 {
    for opt in minus {
        //add
        if !info.o_options.contains(opt) {
            info.o_options.push(opt.to_string());
        }
    }

    for opt in plus {
        //remove
        info.o_options.retain(|e| e != opt);
    }

    0
}

fn compopt_print(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let optlist = vec![
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
            match info.o_options.contains(opt) {
                true => print!("-o {} ", opt),
                false => print!("+o {} ", opt),
            }
        }
        println!("{}", &com);
    } else {
        eprintln!("sush: compopt: {}: no completion specification", &args[1]);
        return 1;
    }

    0
}

pub fn compopt(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        dbg!("{:?}", &core.completion.entries);
        return 1;
    }

    if !args[1].starts_with("-") && !args[1].starts_with("+") {
        return compopt_print(core, args);
    }

    let mut flag = "".to_string();
    let mut minus = vec![];
    let mut plus = vec![];
    let mut minus_d = vec![];
    let mut plus_d = vec![];
    let mut minus_e = vec![];
    let mut plus_e = vec![];

    while args.len() > 1 {
        if args[1] == "-D" || args[1] == "-E" {
            flag = args[1].clone();
            args.remove(1);
            continue;
        }

        if args[1] == "-o" {
            let opt = arg::consume_with_next_arg("-o", args);
            if opt.is_none() {
                return 1;
            }

            match flag.as_str() {
                "" => minus.push(opt.unwrap()),
                "-D" => minus_d.push(opt.unwrap()),
                "-E" => minus_e.push(opt.unwrap()),
                _ => return 1,
            }
            continue;
        }

        if args[1] == "+o" {
            let opt = arg::consume_with_next_arg("+o", args);
            if opt.is_none() {
                return 1;
            }

            match flag.as_str() {
                "" => plus.push(opt.unwrap()),
                "-D" => plus_d.push(opt.unwrap()),
                "-E" => plus_e.push(opt.unwrap()),
                _ => return 1,
            }
            continue;
        }

        break;
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
    return compopt_set(info, &plus, &minus);

    //TODO: support of -D -E
}
