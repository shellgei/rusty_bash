//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod brace_expansion;

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct Word {
    pub text: String,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl Word {
    pub fn eval(&mut self, core: &mut ShellCore) -> Vec<String> {
        let mut ws = brace_expansion::eval(self);

        ws.iter_mut().for_each(|w| w.parameter_expansion(core));
        ws.iter_mut().for_each(|w| w.unquote());
        ws.iter_mut().for_each(|w| w.connect_subwords());
        ws.iter().map(|w| w.text.clone()).filter(|arg| arg.len() > 0).collect()
    }

    fn parameter_expansion(&mut self, core: &mut ShellCore) {
        self.subwords.iter_mut().for_each(|w| w.parameter_expansion(core));
    }

    fn unquote(&mut self) {
        self.subwords.iter_mut().for_each(|w| w.unquote());
    }
    
    fn connect_subwords(&mut self) {
        self.text = self.subwords.iter()
                    .map(|s| s.get_text())
                    .collect::<String>();
    }

    pub fn new() -> Word {
        Word {
            text: String::new(),
            subwords: vec![],
        }
    }

    fn push(&mut self, subword: &Box<dyn Subword>) {
        self.text += &subword.get_text().to_string();
        self.subwords.push(subword.clone());
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Word> {
        if feeder.starts_with("#") {
            return None;
        }

        let mut ans = Word::new();
        while let Some(sw) = subword::parse(feeder, core) {
            ans.push(&sw);
        }

        if ans.text.len() == 0 {
            None
        }else{
            Some(ans)
        }
    }
}
