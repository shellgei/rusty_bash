//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::subword::unquoted::UnquotedSubword;

#[derive(Debug)]
pub struct Word {
    pub text: String,
    pub subwords: Vec<UnquotedSubword>,
}

impl Word {
    pub fn new() -> Word {
        Word {
            text: String::new(),
            subwords: vec![],
        }
    }

    pub fn get_args(&mut self) -> Vec<String> {
        vec![self.text.clone()]
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Word> {
        let mut ans = Word::new();
        while let Some(sw) = UnquotedSubword::parse(feeder, core) {
            ans.text += &sw.text.clone();
            ans.subwords.push(sw);
        }

        if ans.text.len() == 0 {
            None
        }else{
            Some(ans)
        }
    }
}
