//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::elements::value::Value;
// use crate::abst_elems::CommandElem;


pub struct Substitution {
    pub text: String,
    pub name: String,
    pub value: Value,
    pub debug: DebugInfo,
}

impl Substitution {
    pub fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> { 
        let mut ans = vec![];
        ans.push(self.name.clone());
        
        let mut v = "".to_string();
        for s in self.value.eval(conf){
            v += &s;
        }
        ans.push(v);

        ans
    }

    pub fn get_text(&self) -> String { self.text.clone() }

    pub fn new(text: &Feeder, name: String, value: Value) -> Substitution{
        Substitution {
            text: name.clone() + "=" + &value.text.clone(),
            name: name, 
            value: value,
            debug: DebugInfo::init(text)
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Substitution> {
        let backup = text.clone();
        let varname_pos = text.scanner_name(0);
        let var_part = text.consume(varname_pos);//VarName::new(text, varname_pos);

        if ! text.starts_with("=") {
            text.rewind(backup);
            return None;
        }
        text.consume(1); // consume of "=" 
 
        if let Some(value_part) = Value::parse(text, conf){
            Some(Substitution::new(text, var_part, value_part))
        }else{ // empty value
            let empty_word = Value::new();
            Some(Substitution::new(text, var_part, empty_word))
        }
    }
}

