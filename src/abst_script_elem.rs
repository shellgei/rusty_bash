//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elem_blankpart::BlankPart;
use crate::elem_setvars::SetVariables;
use crate::elem_pipeline::Pipeline;
use crate::Feeder;
use crate::ShellCore;
use nix::unistd::Pid;
use crate::elem_compound_paren::CompoundParen;

pub trait ScriptElem {
    fn exec(&mut self, _conf: &mut ShellCore) -> Option<Pid> { None }
}

// job or function comment or blank (finally) 
pub fn hand_input_unit(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn ScriptElem>> {
    if text.len() == 0 {
        return None;
    };

    if text.nth(0) == ')' {
        eprintln!("Unexpected symbol: )");
        return None;
    }

    if let Some(result) = CompoundParen::parse(text, conf) {return Some(Box::new(result));}
    if let Some(result) = BlankPart::parse(text)           {return Some(Box::new(result));}
    if let Some(result) = SetVariables::parse(text, conf)  {return Some(Box::new(result));}
    if let Some(result) = Pipeline::parse(text, conf)      {return Some(Box::new(result));}

    eprintln!("Unknown phrase");
    None
}
