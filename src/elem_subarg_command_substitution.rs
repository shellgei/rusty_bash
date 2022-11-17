//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;

use crate::abst_elems::ArgElem;
use crate::abst_elems::Compound;
use crate::elem_compound_paren::CompoundParen;

pub struct SubArgCommandSubstitution {
    pub text: String,
    pub pos: DebugInfo,
    pub com: CompoundParen, 
    pub is_value: bool,
}

impl ArgElem for SubArgCommandSubstitution {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<Vec<String>> {
        self.com.substitution = true;
        self.com.exec(conf);

        if self.is_value {
            return vec!(vec!(self.com.substitution_text.clone()));
        }

        let ans = self.com.substitution_text
                .split(" ")
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

        vec!(ans)
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgCommandSubstitution {
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, is_value: bool) -> Option<SubArgCommandSubstitution> {
        if text.len() == 0 || text.nth(0) != '$' {
            return None;
        }
    
        let backup = text.clone();
        text.consume(1);

        if let Some(e) = CompoundParen::parse(text, conf, true){
            let ans = SubArgCommandSubstitution {
                text: "$".to_owned() + &e.get_text(),
                pos: DebugInfo::init(text),
                com: e,
                is_value: is_value};
    
            return Some(ans);
        }else{
            text.rewind(backup);
            None
        }
    }
}
