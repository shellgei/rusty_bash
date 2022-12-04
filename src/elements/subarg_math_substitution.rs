//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;

use crate::abst_elems::ArgElem;
use crate::abst_elems::compound::Compound;
use crate::elements::compound_double_paren::CompoundDoubleParen;

pub struct SubArgMathSubstitution {
    pub text: String,
    pub pos: DebugInfo,
    pub com: CompoundDoubleParen, 
    //pub is_value: bool,
}

impl ArgElem for SubArgMathSubstitution {
    fn eval(&mut self, conf: &mut ShellCore, _as_value: bool) -> Vec<Vec<String>> {
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

impl SubArgMathSubstitution {
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore/*, is_value: bool*/) -> Option<SubArgMathSubstitution> {
        if ! text.starts_with("$") {
            return None;
        }
    
        let backup = text.clone();
        text.consume(1);

        if let Some(e) = CompoundDoubleParen::parse(text, conf, true){
            let ans = SubArgMathSubstitution {
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
