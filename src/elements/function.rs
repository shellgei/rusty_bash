//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::varname::VarName;
//use crate::feeder::scanner::*;
use crate::abst_elems::compound;
use crate::abst_elems::Compound;

pub struct Function {
    pub name: String,
    pub body: Box<dyn Compound>,
    pub text: String,
}

impl Function {
    pub fn new(name: String, body: Box<dyn Compound>, text: String) -> Function{
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

         loop { //remove keyword function
             let var_pos = text.scanner_name(0);
             if var_pos == 0 {
                 text.rewind(backup);
                 return None;
             }
             name = VarName::new(text, var_pos);

             let d = text.scanner_blank(0);
             ans_text += &text.consume(d);

             if name.text != "function" {
                 break;
             }
         }

         if ! text.starts_with("(") {
             text.rewind(backup);
             return None;
         }
         ans_text += &text.consume(1);
         let d = text.scanner_blank(0);
         ans_text += &text.consume(d);
 
         if ! text.starts_with(")") {
             text.rewind(backup);
             return None;
         }
         ans_text += &text.consume(1);
 
         let d = text.scanner_blank(0);
         ans_text += &text.consume(d);
 
         if let Some(c) = compound(text, conf){
             Some( Function::new(name.text, c, ans_text) )
         }else{
             text.rewind(backup);
             None
         }
    }
}
