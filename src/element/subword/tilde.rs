//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
//use crate::feeder::scanner::*;
use crate::utils::expand_tilde;

use crate::element::subword::WordElem;

pub struct SubWordTildePrefix {
    pub text: String,
    pub pos: DebugInfo,
}

impl WordElem for SubWordTildePrefix {
    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn eval(&mut self, _conf: &mut ShellCore, _: bool) -> Vec<Vec<String>> {
        vec!(vec!(expand_tilde(&self.text).0))
    }
}

impl SubWordTildePrefix {
    pub fn parse(text: &mut Feeder, _: bool) -> Option<SubWordTildePrefix> {
        let pos = text.scanner_tilde_prefix();
        if pos != 0 {
            Some( SubWordTildePrefix{text: text.consume(pos), pos: DebugInfo::init(text) } )
        }else{
            None
        }
    }
}
