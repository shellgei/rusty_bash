//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elem_varname::VarName;
use crate::scanner::*;
use crate::abst_elems::PipelineElem;
use crate::abst_elems::compound;

pub struct Function {
    pub name: String,
    pub body: Box<dyn PipelineElem>,
    pub text: String,
}

impl Function {
    pub fn new(name: String, body: Box<dyn PipelineElem>, text: String) -> Function{
        Function {
            name: name,
            body: body,
            text: text,
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Function> {
         let backup = text.clone();
         let mut name;
         let mut ans_text = String::new();

         loop {
             let var_pos = scanner_varname(text, 0);
             if var_pos == 0 {
                 text.rewind(backup);
                 return None;
             }
             name = VarName::new(text, var_pos);

             let d = scanner_while(text, 0, " \t");
             ans_text += &text.consume(d);

             if name.text != "function" {
                 break;
             }
         }

         if text.len() == 0 || text.nth(0) != '(' {
             text.rewind(backup);
             return None;
         }
         ans_text += &text.consume(1);
         let d = scanner_while(text, 0, " \t");
         ans_text += &text.consume(d);
 
         if text.len() == 0 || text.nth(0) != ')' {
             text.rewind(backup);
             return None;
         }
         ans_text += &text.consume(1);
 
         let d = scanner_while(text, 0, " \t");
         ans_text += &text.consume(d);
 
         if let Some(c) = compound(text, conf){
             Some( Function::new(name.text, c, ans_text) )
         }else{
             text.rewind(backup);
             None
         }
    }
}
