//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::elem_arg::Arg;
use crate::elems_in_arg::VarName;
use crate::scanner::scanner_varname;
use crate::scanner::scanner_until;
use crate::parser_args::arg;
use crate::ElemOfCommand;


pub struct Substitution {
    pub text: String,
    pub name: VarName,
    pub value: Arg,
    pub debug: DebugInfo,
}

impl Substitution {
    pub fn new(text: &Feeder, name: VarName, value: Arg) -> Substitution{
        Substitution {
            text: name.text.clone() + "=" + &value.text.clone(),
            name: name, 
            value: value,
            debug: DebugInfo::init(text)
        }
    }

    pub fn parse(text: &mut Feeder) -> Option<Substitution> {
        let varname_pos = scanner_varname(text, 0);
        let equal_pos = scanner_until(text, varname_pos, "=");
        if equal_pos != varname_pos {
            return None;
        }
        if equal_pos == text.len() {
            return None;
        }
    
        let backup = text.clone();
        let var_part = VarName::new(text, varname_pos);
        text.consume(1); // = 
        if let Some(value_part) = arg(text, false){
            Some(Substitution::new(text, var_part, value_part))
        }else{ // cases where the value goes the next line
            text.rewind(backup);
            None
        }
    }
}

impl ElemOfCommand for Substitution {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    substitution: '{}' ({})\n", self.text.clone(), self.debug.text()))
    }

    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> { 
        let mut ans = vec!();
        ans.push(self.name.text.clone());
        
        let mut v = "".to_string();
        for s in self.value.eval(conf){
            v += &s;
        }
        ans.push(v);

        ans
    }

    fn text(&self) -> String { self.text.clone() }
}

