//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::elems_executable::{Substitutions, Executable, BlankPart, CommandWithArgs};
use super::elems_in_command::{ArgDelimiter, Eoc};
use crate::parser_args::{arg, substitution, redirect};
use crate::Feeder;
use crate::debuginfo::DebugInfo;
use crate::scanner::{scanner_end_of_com, scanner_while};

// job or function comment or blank (finally) 
pub fn top_level_element(text: &mut Feeder) -> Option<Box<dyn Executable>> {
    if text.len() == 0 {
        return None;
    };

    if let Some(result) = blank_part(text)       {return Some(Box::new(result));}
    if let Some(result) = substitutions(text)    {return Some(Box::new(result));}
    if let Some(result) = command_with_args(text){return Some(Box::new(result));}

    if text.error_occuring {
        text.consume(text.len());
        eprintln!("{}", text.error_reason);
        text.error_occuring = false;
    };
    None
}

pub fn blank_part(text: &mut Feeder) -> Option<BlankPart> {
    let mut ans = BlankPart::new();

    loop {
        if let Some(d) = delimiter(text)          {ans.push(Box::new(d));}
        else if let Some(e) = end_of_command(text){ans.push(Box::new(e));}
        else{break;};
    };

    BlankPart::return_if_valid(ans)
}

pub fn substitutions(text: &mut Feeder) -> Option<Substitutions> {
    let backup = text.clone();
    let mut ans = Substitutions::new();

    while let Some(result) = substitution(text) {
        ans.push(Box::new(result));

        if let Some(result) = delimiter(text){
            ans.push(Box::new(result));
        }
    }

    if let Some(result) = end_of_command(text){
        ans.push(Box::new(result));
    }else{
        text.rewind(backup);
        return None;
    }

    Substitutions::return_if_valid(ans)
}


pub fn command_with_args(text: &mut Feeder) -> Option<CommandWithArgs> {
    let backup = text.clone();
    let mut ans = CommandWithArgs::new();

    //TODO: bash permits redirections here. 

    /* A command starts with substitutions. */
    while let Some(s) = substitution(text) {
        ans.push_vars(s);

        if let Some(d) = delimiter(text){
            ans.push_elems(Box::new(d));
        }
    }

    //TODO: bash permits redirections here. 

    /* Then one or more arguments exist. */
    while let Some(a) = arg(text, true) {
        if text.len() != 0 {
            if text.nth(0) == ')' || text.nth(0) == '(' {
                text.error_occuring = true;
                text.error_reason = "Unexpected token found".to_string();
                text.rewind(backup);
                return None;
            };
        };
        ans.push_elems(Box::new(a));

        if let Some(d) = delimiter(text){
            ans.push_elems(Box::new(d));
        }

        /* When a redirect is found. The command ends with redirects. */
        if let Some(r) = redirect(text){
            ans.redirects.push(Box::new(r));
            while let Some(r) = redirect(text){
                ans.redirects.push(Box::new(r));
            }
            break;
        }

        if text.len() == 0 {
            break;
        }

        if let Some(e) = end_of_command(text){
            ans.push_elems(Box::new(e));
            break;
        }
    }

    CommandWithArgs::return_if_valid(ans, text, backup)
}

pub fn delimiter(text: &mut Feeder) -> Option<ArgDelimiter> {
    let pos = scanner_while(text, 0, " \t");
    ArgDelimiter::return_if_valid(text, pos)
}

pub fn end_of_command(text: &mut Feeder) -> Option<Eoc> {
    if text.len() == 0 {
     //   return Some(Eoc{text: "".to_string(), debug: DebugInfo::init(&text)});
        return None;
    };

    let pos = scanner_end_of_com(text, 0);
    if pos == 0 {
        return None;
    };

    Some(Eoc{text: text.consume(pos), debug: DebugInfo::init(&text)})
}
