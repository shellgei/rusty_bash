//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
//use crate::feeder::scanner::*;
use crate::utils;

use crate::elements::subword::Subword;

#[derive(Debug)]
pub struct SubwordTildePrefix {
    pub text: String,
    pub pos: DebugInfo,
}

impl Subword for SubwordTildePrefix {
    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn eval(&mut self, _conf: &mut ShellCore, _: bool) -> Vec<Vec<String>> {
        vec!(vec!(utils::tilde_to_dir(&self.text).0))
    }
}

impl SubwordTildePrefix {
    pub fn parse(text: &mut Feeder, _: bool) -> Option<SubwordTildePrefix> {
        let pos = text.scanner_tilde_prefix();
        if pos != 0 {
            Some( SubwordTildePrefix{text: text.consume(pos), pos: DebugInfo::init(text) } )
        }else{
            None
        }
    }
}
