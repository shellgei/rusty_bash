//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;

use crate::abst_arg_elem::ArgElem;
use crate::abst_script_elem::ScriptElem;
use crate::elem_compound_paren::CompoundParen;

pub struct SubArgCommandSubstitution {
    pub text: String,
    pub pos: DebugInfo,
    pub com: CompoundParen, 
}

impl ArgElem for SubArgCommandSubstitution {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<Vec<String>> {
        self.com.substitution = true;
        self.com.exec(conf);

        let ans = self.com.substitution_text
                .split(" ")
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

        vec!(ans)
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgCommandSubstitution {
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<SubArgCommandSubstitution> {
        if text.len() == 0 || text.nth(0) != '$' {
            return None;
        }
    
        let backup = text.clone();
        text.consume(1);

        if let Some(e) = CompoundParen::parse(text, conf, true){
            //e.substitution = true;
            let ans = SubArgCommandSubstitution {
                text: "$".to_owned() + &e.text.clone(),
                pos: DebugInfo::init(text),
                com: e };
    
            return Some(ans);
        }else{
            text.rewind(backup);
            None
        }
    }
}
