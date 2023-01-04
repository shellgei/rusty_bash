//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
//use crate::feeder::scanner::*;

use crate::elements::subword::Subword;

#[derive(Debug)]
pub struct SubwordStringNonQuoted {
    pub text: String,
    pub pos: DebugInfo,
    //pub is_value: bool,
}

impl Subword for SubwordStringNonQuoted {
    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn eval(&mut self, _conf: &mut ShellCore, _: bool) -> Vec<Vec<String>> {
        /*
        if self.is_value {
            vec!(vec!(self.text.clone()))
        }else{
        */
            vec!(vec!(self.text.replace("\n", " ")))
        //}
    }
}

impl SubwordStringNonQuoted {
    fn new(text: String, pos: DebugInfo/*, is_value: bool*/) -> SubwordStringNonQuoted {
        SubwordStringNonQuoted {
            text: text.clone(),
            pos: pos,
            //is_value: is_value, 
        }
    }

    pub fn parse(text: &mut Feeder, is_in_brace: bool, ignore_brace: bool) -> Option<SubwordStringNonQuoted> {
        let pos = text.scanner_non_quoted_word(is_in_brace, ignore_brace);
        if pos == 0{
            None
        }else{
            Some( SubwordStringNonQuoted::new(text.consume(pos), DebugInfo::init(text)/*, false*/) )
        }
    }
}
