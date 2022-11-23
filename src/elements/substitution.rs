//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::elements::arg::Arg;
use crate::elements::varname::VarName;
use crate::scanner::scanner_name;
use crate::abst_elems::CommandElem;


pub struct Substitution {
    pub text: String,
    pub name: VarName,
    pub value: Arg,
    pub debug: DebugInfo,
}

impl CommandElem for Substitution {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    substitution: '{}' ({})\n", self.text.clone(), self.debug.get_text()))
    }

    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> { 
        let mut ans = vec![];
        ans.push(self.name.text.clone());
        
        let mut v = "".to_string();
        for s in self.value.eval(conf){
            v += &s;
        }
        ans.push(v);

        ans
    }

    fn get_text(&self) -> String { self.text.clone() }
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

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Substitution> {
        let backup = text.clone();
        let varname_pos = scanner_name(text, 0);
        let var_part = VarName::new(text, varname_pos);

        if ! text.starts_with("=") {
            text.rewind(backup);
            return None;
        }
        text.consume(1); // consume of "=" 
 
        if let Some(value_part) = Arg::parse(text, conf, true, false){
            Some(Substitution::new(text, var_part, value_part))
        }else{ // empty value
            let empty_arg = Arg::new();
            Some(Substitution::new(text, var_part, empty_arg))
        }
    }
}

