//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;

use crate::element::subword::Subword;
use crate::element::command::Command;
use crate::element::command::paren::CommandParen;

pub struct SubwordCommandSubstitution {
    pub text: String,
    pub pos: DebugInfo,
    pub com: CommandParen, 
//    pub is_value: bool,
}

impl Subword for SubwordCommandSubstitution {
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

impl SubwordCommandSubstitution {
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore/*, is_value: bool*/) -> Option<SubwordCommandSubstitution> {
        if ! text.starts_with("$") {
            return None;
        }
    
        let backup = text.clone();
        text.consume(1);

        if let Some(e) = CommandParen::parse(text, conf, true){
            let ans = SubwordCommandSubstitution {
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
