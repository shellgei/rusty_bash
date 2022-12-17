//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;

use crate::element::subword::WordElem;
use crate::element::command::AbstCommand;
use crate::element::command::double_paren::CommandDoubleParen;

pub struct SubWordMathSubstitution {
    pub text: String,
    pub pos: DebugInfo,
    pub com: CommandDoubleParen, 
    //pub is_value: bool,
}

impl WordElem for SubWordMathSubstitution {
    fn eval(&mut self, conf: &mut ShellCore, _: bool) -> Vec<Vec<String>> {
        self.com.substitution = true;
        self.com.exec(conf);

//        if self.is_value {
            return vec!(vec!(self.com.substitution_text.clone()));
 //       }

            /*
        let ans = self.com.substitution_text
                .split(" ")
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

        vec!(ans)
        */
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
}

impl SubWordMathSubstitution {
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore/*, is_value: bool*/) -> Option<SubWordMathSubstitution> {
        if ! text.starts_with("$") {
            return None;
        }
    
        let backup = text.clone();
        text.consume(1);

        if let Some(e) = CommandDoubleParen::parse(text, conf, true){
            let ans = SubWordMathSubstitution {
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
