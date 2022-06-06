//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::elem_command::{Executable};
use crate::elem_blankpart::BlankPart;
use crate::elem_setvars::SetVariables;
use crate::elem_command::Command;
use crate::Feeder;

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
