//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elem_arg_delimiter::ArgDelimiter;
use crate::elem_compound_brace::CompoundBrace;
use crate::elem_varname::VarName;
use crate::scanner::scanner_varname;

pub struct Function {
    pub name: String,
    pub body: CompoundBrace,
    pub text: String,
}

impl Function {
    pub fn new(name: String, body: CompoundBrace) -> Function{
        Function {
            name: name,
            body: body,
            text: "".to_string(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Function> {
         let var_pos = scanner_varname(text, 0);
         if var_pos == 0 {
             return None;
         }

         let backup = text.clone();
         let name = VarName::new(text, var_pos);

        let _ = ArgDelimiter::parse(text);

        if text.len() == 0 || text.nth(0) != '(' {
            text.rewind(backup);
            return None;
        }
        text.consume(1);

        if text.len() == 0 || text.nth(0) != ')' {
            text.rewind(backup);
            return None;
        }
        text.consume(1);

        let _ = ArgDelimiter::parse(text);

        if let Some(c) = CompoundBrace::parse(text, conf){
            Some( Function::new(name.text, c) )
        }else{
            text.rewind(backup);
            None
        }
    }
}
