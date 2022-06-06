//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::elem_command::{Executable};
use super::elems_in_command::{ArgDelimiter, Eoc};
use crate::Feeder;
use crate::debuginfo::DebugInfo;
use crate::scanner::{scanner_end_of_com, scanner_while};

use crate::elem_blankpart::blank_part;
use crate::elem_substitutions::substitutions;
use crate::elem_command::command_with_args;

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
