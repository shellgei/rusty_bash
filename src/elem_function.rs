//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elem_arg_delimiter::ArgDelimiter;
use crate::elem_compound_brace::CompoundBrace;
use crate::elem_compound_paren::CompoundParen;
use crate::elem_varname::VarName;
use crate::scanner::scanner_varname;
use crate::List;

pub struct Function {
    pub name: String,
    pub body: Box<dyn List>,
    pub text: String,
}

impl Function {
    pub fn new(name: String, body: Box<dyn List>) -> Function{
        Function {
            name: name,
            body: body,
            text: "".to_string(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Function> {
         let backup = text.clone();
         let mut name;

         loop {
             let var_pos = scanner_varname(text, 0);
             if var_pos == 0 {
                 text.rewind(backup);
                 return None;
             }
             name = VarName::new(text, var_pos);
             let _ = ArgDelimiter::parse(text);

             if name.text != "function" {
                 break;
             }
         }

         if text.len() == 0 || text.nth(0) != '(' {
             text.rewind(backup);
             return None;
         }
         text.consume(1);
         let _ = ArgDelimiter::parse(text);
 
         if text.len() == 0 || text.nth(0) != ')' {
             text.rewind(backup);
             return None;
         }
         text.consume(1);
 
         let _ = ArgDelimiter::parse(text);
 
         if let Some(c) = CompoundBrace::parse(text, conf){
             Some( Function::new(name.text, Box::new(c)) )
         }else if let Some(c) = CompoundParen::parse(text, conf, true){
             Some( Function::new(name.text, Box::new(c)) )
         }else{
             text.rewind(backup);
             None
         }
    }
}
