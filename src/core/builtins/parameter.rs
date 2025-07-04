//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{env, ShellCore};
use crate::error::exec::ExecError;
use crate::elements::substitution::Substitution;
use crate::utils::arg;
use super::error_exit;

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

fn set_substitution(core: &mut ShellCore, sub: &mut Substitution, args: &mut Vec<String>,
                    layer: usize) -> Result<(), ExecError> {
    if core.db.is_readonly(&sub.left_hand.name) {
        return Err(ExecError::VariableReadOnly(sub.left_hand.name.clone()));
    }

    let read_only = arg::consume_option("-r", args);
    let export_opt = arg::consume_option("-x", args);
    let little_opt = arg::consume_option("-l", args);

    let mut layer = layer;
    if arg::consume_option("-g", args) && layer != 0 {
        core.db.unset(&sub.left_hand.name);
        layer = 0;
    }

    if arg::consume_option("+i", args) {
        if core.db.has_flag_layer(&sub.left_hand.name, 'i', layer) {
            core.db.int_to_str_type(&sub.left_hand.name, layer)?;
        }
    }

    if ( args.contains(&"-A".to_string()) || args.contains(&"-a".to_string()) )
    && ! core.db.exist(&sub.left_hand.name) {
        sub.left_hand.init_variable(core, Some(layer), args)?;
    }

    if sub.has_right /*&& sub.left_hand.index.is_none()*/ { //TODO: ???????
        if (args.contains(&"-a".to_string()) || args.contains(&"-A".to_string())) 
        || (core.db.is_array(&sub.left_hand.name) || core.db.is_assoc(&sub.left_hand.name) ) {

            if ! (sub.left_hand.index.is_some() && sub.right_hand.text.starts_with("'") ) {
                sub.reparse(core)?; //TODO: reparate reparse of variable and value from each other
            }
        }
    }

    if export_opt {
        core.db.set_flag(&sub.left_hand.name, 'x', Some(layer));
    }

    if args.contains(&"-i".to_string()) {
        core.db.set_flag(&sub.left_hand.name, 'i', Some(layer));
    }

    if little_opt {
        core.db.set_flag(&sub.left_hand.name, 'l', Some(layer));
    }

    let mut res = Ok(());
 
    match sub.has_right {
        true  => res = sub.eval(core, Some(layer), true),
        false => {
            if ! core.db.params[layer].contains_key(&sub.left_hand.name)
            || ( ! core.db.is_array(&sub.left_hand.name) && args.contains(&"-a".to_string()) )
            || ( ! core.db.is_assoc(&sub.left_hand.name) && args.contains(&"-A".to_string()) ) {
                res = sub.left_hand.init_variable(core, Some(layer), args);
            }
        },
    }

    if read_only {
        core.db.set_flag(&sub.left_hand.name, 'r', Some(layer));
    }

    res
}

fn declare_print(core: &mut ShellCore, names: &[String], com: &str) -> i32 {
    for n in names {
        let mut opt = if core.db.is_assoc(&n) { "A" }
        else if core.db.is_array(&n) { "a" }
        else if core.db.exist(&n) { "" }
        else{
            return error_exit(1, &n, "not found", core);
        }.to_string();

        if core.db.is_int(&n) {
                opt += "i";
        }

        if core.db.has_flag(&n, 'l') {
                opt += "l";
        }

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
        core.db.declare_print(&n);
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
        for sub in subs {
            args.push(sub.text.clone());
        }
        return declare_print(core, &args[1..], &args[0]);
    }

    let layer = core.db.get_layer_num() - 2;
    for sub in subs {
        if let Err(e) = set_substitution(core, sub, &mut args.clone(), layer) {
            return super::error_exit(1, &args[0], &String::from(&e), core);
            /*
            e.print(core);
            return 1;
            */
        }
    }
    0
}

pub fn export(core: &mut ShellCore, args: &mut Vec<String>,
              subs: &mut Vec<Substitution>) -> i32 {
    for sub in subs.iter_mut() {
        let layer = core.db.get_layer_pos(&sub.left_hand.name).unwrap_or(0);
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

    for sub in subs {
        if sub.left_hand.index.is_some() {
            let msg = ExecError::VariableInvalid(sub.left_hand.text.clone());
            return super::error_exit(1, &args[0], &String::from(&msg), core);
            //return 1;
        }

        let layer = core.db.get_layer_pos(&sub.left_hand.name).unwrap_or(0);

        if let Err(e) = set_substitution(core, sub, &mut args.clone(), layer) {
            e.print(core);
            return 1;
        }
        core.db.set_flag(&sub.left_hand.name, 'r', Some(layer));
    }
    0
}
