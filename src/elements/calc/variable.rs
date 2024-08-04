//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::CalcElement;
use super::syntax_error_msg;
use crate::elements::calc::Word;

fn is_name(s: &str, core: &mut ShellCore) -> bool {
    let mut f = Feeder::new(s);
    s.len() > 0 && f.scanner_name(core) == s.len()
}

fn recursion_error(token: &str) -> String {
    format!("{0}: expression recursion level exceeded (error token is \"{0}\")", token)
}

fn str_to_num(name: &str, core: &mut ShellCore) -> Result<i64, String> {
    let mut name = name.to_string();

    const RESOLVE_LIMIT: i32 = 10000;

    for i in 0..RESOLVE_LIMIT {
        match is_name(&name, core) {
            true  => name = core.data.get_param(&name),
            false => break,
        }

        if i == RESOLVE_LIMIT - 1 {
            return Err(recursion_error(&name));
        }
    }

    if let Ok(n) = name.parse::<i64>() {
        Ok( n )
    }else if name == "" || is_name(&name, core) {
        Ok( 0 )
    }else{
        Err(syntax_error_msg(&name))
    }
}

pub fn word_to_operand(w: &Word, pre_increment: i64, post_increment: i64,
                   core: &mut ShellCore) -> Result<CalcElement, String> {
    if pre_increment != 0 && post_increment != 0 
    || w.text.find('\'').is_some() {
        return Err(syntax_error_msg(&w.text));
    }

    let name = match w.eval_as_value(core) {
        Some(v) => v, 
        None => return Err(format!("{}: wrong substitution", &w.text)),
    };

    let res = match pre_increment {
        0 => word_to_i64(&name, core, post_increment, false),
        _ => word_to_i64(&name, core, pre_increment, true),
    };

    match res {
        Ok(n)  => return Ok(CalcElement::Operand(n)),
        Err(e) => return Err(e),
    }
}

fn word_to_i64(name: &str, core: &mut ShellCore, inc: i64, pre: bool) -> Result<i64, String> {
    if ! is_name(name, core) {
        return match inc != 0 && ! pre {
            true  => Err(syntax_error_msg(name)),
            false => str_to_num(&name, core),
        }
    }

    let num_i64 = match str_to_num(&name, core) {
        Ok(n)        => n,
        Err(err_msg) => return Err(err_msg), 
    };
    
    core.data.set_param(name, &(num_i64 + inc).to_string());

    match pre {
        true  => Ok(num_i64+inc),
        false => Ok(num_i64),
    }
}

pub fn inc(inc: i64, stack: &mut Vec<CalcElement>, core: &mut ShellCore) -> Result<(), String> {
    match stack.pop() {
        Some(CalcElement::Word(w, inc_post)) => {
            match word_to_operand(&w, inc, inc_post, core) {
                Ok(op) => {
                    stack.push(op);
                    Ok(())
                },
                Err(e) => Err(e),
            //    _      => Err("unknown word parse error".to_string()),
            }
        },
        _ => Err("invalid increment".to_string()),
    }
}
