//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::elem_command::{Executable};
use super::elem_arg::{ArgDelimiter, Eoc};
use crate::elem_blankpart::BlankPart;
use crate::elem_setvars::SetVariables;
use crate::elem_command::Command;
use crate::Feeder;
use crate::debuginfo::DebugInfo;
use crate::scanner::{scanner_end_of_com, scanner_while};

// job or function comment or blank (finally) 
pub fn top_level_element(text: &mut Feeder) -> Option<Box<dyn Executable>> {
    if text.len() == 0 {
        return None;
    };

    if let Some(result) = BlankPart::parse(text)    {return Some(Box::new(result));}
    if let Some(result) = SetVariables::parse(text) {return Some(Box::new(result));}
    if let Some(result) = Command::parse(text)      {return Some(Box::new(result));}

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
        return None;
    };

    let pos = scanner_end_of_com(text, 0);
    if pos == 0 {
        return None;
    };

    Some(Eoc{text: text.consume(pos), debug: DebugInfo::init(&text)})
}
