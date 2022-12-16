//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
//use crate::feeder::scanner::*;
use crate::abst_elems::command;
use crate::abst_elems::command::Compound;

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
         let mut ans_text = String::new();

         if text.starts_with("function") {
            ans_text += &text.consume(8);
            ans_text += &text.consume_blank();
         }

         let var_pos = text.scanner_name(0);
         if var_pos == 0 {
             text.rewind(backup);
             return None;
         }
         let name = text.consume(var_pos);
         ans_text += &text.consume_blank();


         if ! text.starts_with("(") {
             text.rewind(backup);
             return None;
         }
         ans_text += &text.consume(1);
         ans_text += &text.consume_blank();
 
         if ! text.starts_with(")") {
             text.rewind(backup);
             return None;
         }
         ans_text += &text.consume(1);
         ans_text += &text.consume_blank();
 
         if let Some(c) = command::parse(text, conf){
             Some( Function::new(name, c, ans_text) )
         }else{
             text.rewind(backup);
             None
         }
    }
}
