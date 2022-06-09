//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elem_blankpart::BlankPart;
use crate::elem_setvars::SetVariables;
use crate::elem_pipeline::Pipeline;
use crate::Feeder;
use crate::ShellCore;

pub trait HandInputUnit {
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String> { vec!() }
    fn exec(&mut self, _conf: &mut ShellCore) -> String { "".to_string() }
}

// job or function comment or blank (finally) 
pub fn hand_input_unit(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn HandInputUnit>> {
    if text.len() == 0 {
        return None;
    };

    if let Some(result) = BlankPart::parse(text)          {return Some(Box::new(result));}
    if let Some(result) = SetVariables::parse(text, conf) {return Some(Box::new(result));}
    if let Some(result) = Pipeline::parse(text, conf)     {return Some(Box::new(result));}

    if text.error_occuring {
        text.consume(text.len());
        eprintln!("{}", text.error_reason);
        text.error_occuring = false;
    };
    None
}
