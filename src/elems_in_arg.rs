//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
use crate::elem_command::{Command};
use crate::scanner::*;

use crate::abst_elem_argelem::ArgElem;
use crate::elem_script::Executable;

pub struct SubArgVariable {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgVariable {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let name = if self.text.rfind('}') == Some(self.text.len()-1) {
            self.text[2..self.text.len()-1].to_string()
        }else{
            self.text[1..].to_string()
        };
        vec!(conf.get_var(&name))
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgVariable {
    pub fn parse(text: &mut Feeder) -> Option<SubArgVariable> {
        if !(text.nth(0) == '$') || text.nth(1) == '{' {
            return None;
        };
    
        let pos = scanner_varname(&text, 1);
        Some(
            SubArgVariable{
                text: text.consume(pos),
                pos: DebugInfo::init(text),
            })
    }
    
    pub fn parse2(text: &mut Feeder) -> Option<SubArgVariable> {
        if !(text.nth(0) == '$' && text.nth(1) == '{') {
            return None;
        }
    
        let pos = scanner_varname(&text, 2);
        if text.nth(pos) == '}' {
            Some( SubArgVariable{ text: text.consume(pos+1), pos: DebugInfo::init(text) })
        }else{
            None
        }
    }
}

pub struct SubArgCommandExp {
    pub text: String,
    pub pos: DebugInfo,
    pub com: Command, 
}

impl ArgElem for SubArgCommandExp {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        self.com.expansion = true;
        vec!(self.com.exec(conf).replace("\n", " "))
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgCommandExp {
    pub fn parse(text: &mut Feeder) -> Option<SubArgCommandExp> {
        if !(text.nth(0) == '$' && text.nth(1) == '(') {
            return None;
        }
    
        let pos = scanner_end_of_bracket(text, 2, ')');
        let mut sub_feeder = Feeder::new_with(text.from_to(2, pos));
    
        if let Some(e) = Command::parse(&mut sub_feeder){
            let ans = Some (SubArgCommandExp {
                text: text.consume(pos+1),
                pos: DebugInfo::init(text),
                com: e }
            );
    
            return ans;
        };
        None
    }
}


