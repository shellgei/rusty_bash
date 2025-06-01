//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{env, ShellCore, Feeder};
use crate::error::exec::ExecError;
use crate::elements::substitution::Substitution;
use crate::elements::word::Word;
use crate::utils::arg;
use super::error_exit;

pub fn set_positions(core: &mut ShellCore, args: &[String]) -> Result<(), ExecError> {
    if core.db.position_parameters.pop().is_none() {
        return Err(ExecError::Other("empty param stack".to_string()));
    }
    core.db.position_parameters.push(args.to_vec());
    Ok(())
}

pub fn local(core: &mut ShellCore,
             args: &mut Vec<String>, subs: &mut Vec<Substitution>) -> i32 {
    let layer = if core.db.get_layer_num() > 2 {
        core.db.get_layer_num() - 2//The last element of data.parameters is for local itself. 
    }else{
        ExecError::ValidOnlyInFunction("local".to_string()).print(core);
        return 1;
    };

    for sub in subs.iter_mut() {
        if let Err(e) = set_substitution(core, sub, &mut args.clone(), layer) {
            e.print(core);
            return 1;
        }
    }

    0
}

fn reparse(core: &mut ShellCore, sub: &mut Substitution) {
    let mut f = Feeder::new(&sub.text);
    let text = if let Ok(Some(s)) = Word::parse(&mut f, core, None) {
        if ! f.is_empty() {
            return;
        }

        match s.eval_as_value(core) {
            Ok(txt) => txt,
            _ => return,
        }
    }else{
        return;
    };

    let mut f = Feeder::new(&text.replace("~", "\\~"));
    if let Ok(Some(s)) = Substitution::parse(&mut f, core, false) {
        if ! f.is_empty() {
            return;
        }

        *sub = s;
    }
}

fn set_substitution(core: &mut ShellCore, sub: &mut Substitution, args: &mut Vec<String>,
                    layer: usize) -> Result<(), ExecError> {
    let read_only = arg::consume_option("-r", args);
    let export_opt = arg::consume_option("-x", args);
    let int_opt = arg::consume_option("-i", args);

    if sub.has_right {
        reparse(core, sub);
    }

    if read_only {
        core.db.set_flag(&sub.left_hand.name, 'r');
        return Err(ExecError::VariableReadOnly(sub.left_hand.name.clone()));
    }

    if export_opt {
        core.db.set_flag(&sub.left_hand.name, 'x');
    }

    if int_opt {
        core.db.set_flag(&sub.left_hand.name, 'i');
    }

    match sub.has_right {
        true  => sub.eval(core, Some(layer), true),
        false => sub.left_hand.init_variable(core, Some(layer), args),
    }
}

fn declare_print(core: &mut ShellCore, names: &[String], com: &str) -> i32 {
    for n in names {
        let mut opt = if core.db.is_assoc(&n) { "A" }
        else if core.db.is_array(&n) { "a" }
        else if core.db.has_value(&n) { "" }
        else{
            return error_exit(1, &n, "not found", core);
        }.to_string();

        if core.db.is_readonly(&n) {
            if ! opt.contains('r') 
            && ! core.options.query("posix") {
                opt += "r";
            }
        }

        if opt.is_empty() {
            opt += "-";
        }

        let prefix = match core.options.query("posix") {
            false => format!("declare -{} ", opt),
            true  => format!("{} -{} ", com, opt),
        };
        print!("{}", prefix);
        core.db.print(&n);
    }
    0
}

fn declare_print_all(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        core.db.get_keys().into_iter()
            .for_each(|k| core.db.print(&k));
        return 0;
    }

    if args.len() == 2 && args[1] == "-f" {
        let mut names: Vec<String> = core.db.functions.keys().map(|k| k.to_string()).collect();
        names.sort();

        for n in names {
            core.db.functions.get_mut(&n).unwrap().pretty_print(0); 
        }
        return 0;
    }

    let mut names = core.db.get_keys();
    let mut options = String::new();

    if arg::consume_option("-i", args) {
        names.retain(|n| core.db.has_flag(n, 'i'));
        options += "i";
    }

    if arg::consume_option("-a", args) {
        names.retain(|n| core.db.is_array(n));
        options += "a";
    }

    if arg::consume_option("-A", args) {
        names.retain(|n| core.db.is_assoc(n));
        options += "A";
    }

    if arg::consume_option("-r", args) {
        names.retain(|n| core.db.is_readonly(n));
        if ! core.options.query("posix") {
            options += "r";
        }
    }

    let prefix = format!("declare -{}", options);
    for name in names {
        print!("{}", prefix);
        if core.db.is_readonly(&name)
        && ! options.contains('r')
        && ! core.options.query("posix") {
            print!("r");
        }
        print!(" ");
        core.db.declare_print(&name);
    }
    0
}

pub fn declare(core: &mut ShellCore, args: &mut Vec<String>, subs: &mut Vec<Substitution>) -> i32 {
    let mut args = arg::dissolve_options(args);

    if args[1..].iter().all(|a| a.starts_with("-")) && subs.is_empty() {
        return declare_print_all(core, &mut args);
    }

    if arg::consume_option("-p", &mut args) {
        return declare_print(core, &args[1..], &args[0]);
    }

    for sub in subs {
        let layer = core.db.get_layer_num() - 2;
        if let Err(e) = set_substitution(core, sub, &mut args.clone(), layer) {
            e.print(core);
            return 1;
        }
    }
    0
}

pub fn export(core: &mut ShellCore, args: &mut Vec<String>,
              subs: &mut Vec<Substitution>) -> i32 {

    let layer = core.db.get_layer_num() - 2;//TODO: it is not tested
    for sub in subs.iter_mut() {
        if let Err(e) = set_substitution(core, sub, &mut args.clone(), layer) {
            e.print(core);
            return 1;
        }
        match core.db.get_param(&sub.left_hand.name) {
            Ok(v) => env::set_var(&sub.left_hand.name, v),
            Err(e) => {e.print(core); return 1;},
        }
    }
    0
}

pub fn readonly_print(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let array_opt = arg::consume_option("-a", args);
    let assoc_opt = arg::consume_option("-A", args);
    let int_opt = arg::consume_option("-i", args);

    let mut names: Vec<String> = core.db.get_keys()
        .iter()
        .filter(|e| core.db.is_readonly(&e))
        .map(|e| e.to_string())
        .collect();

    if array_opt { names.retain(|e| core.db.is_array(&e)); }
    if assoc_opt { names.retain(|e| core.db.is_assoc(&e)); }
    if int_opt { names.retain(|e| core.db.is_single_num(&e)); }

    declare_print(core, &names, &args[0]);
    return 0;
}

pub fn readonly(core: &mut ShellCore, args: &mut Vec<String>,
                subs: &mut Vec<Substitution>) -> i32 {
    let mut args = arg::dissolve_options(args);

    if subs.is_empty() {
        return readonly_print(core, &mut args);
    }

    let layer = core.db.get_layer_num() - 2;
    for sub in subs {
        if let Err(e) = set_substitution(core, sub, &mut args.clone(), layer) {
            e.print(core);
            return 1;
        }
        core.db.set_flag(&sub.left_hand.name, 'r');
    }
    0
}
