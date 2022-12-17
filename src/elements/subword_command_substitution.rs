//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;

use crate::elements::abst_subword::WordElem;
use crate::command::AbstCommand;
use crate::elements::compound_paren::CommandParen;

pub struct SubWordCommandSubstitution {
    pub text: String,
    pub pos: DebugInfo,
    pub com: CommandParen, 
//    pub is_value: bool,
}

impl WordElem for SubWordCommandSubstitution {
    fn eval(&mut self, conf: &mut ShellCore, remove_lf: bool) -> Vec<Vec<String>> {
        self.com.substitution = true;
        self.com.exec(conf);

        if ! remove_lf {
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

impl SubWordCommandSubstitution {
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore/*, is_value: bool*/) -> Option<SubWordCommandSubstitution> {
        if ! text.starts_with("$") {
            return None;
        }
    
        let backup = text.clone();
        text.consume(1);

        if let Some(e) = CommandParen::parse(text, conf, true){
            let ans = SubWordCommandSubstitution {
                text: "$".to_owned() + &e.get_text(),
                pos: DebugInfo::init(text),
                com: e,
                /*is_value: is_value*/};
    
            return Some(ans);
        }else{
            text.rewind(backup);
            None
        }
    }
}
