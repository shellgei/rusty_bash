//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};

#[derive(Debug)]
pub struct Word {
    pub text: String,
}

impl Word {
    fn new() -> Word {
        Word {
            text: String::new(),
        }
    }

    pub fn get_args(&mut self) -> Vec<String> {
        vec![self.text.clone()]
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Word> {
        let mut ans = Word::new();
        let arg_len = feeder.scanner_word(core);

        if arg_len > 0 {
            ans.text = feeder.consume(arg_len);
            Some(ans)
        }else{
            None
        }
    }
}

