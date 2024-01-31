//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::subword;
use crate::elements::subword::Subword;
use crate::elements::subword::unquoted::UnquotedSubword;

#[derive(Debug)]
pub struct Word {
    pub text: String,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl Word {
    pub fn new() -> Word {
        Word {
            text: String::new(),
            subwords: vec![],
        }
    }
    pub fn copy(&self) -> Word {
        Word {
            text: self.text.clone(),
            subwords: self.subwords.iter().map(|e| e.copy()).collect(),
        }
    }

    pub fn concat(&mut self, left: &Word) {
        self.text += &left.text;
        for e in left.subwords.iter() {
            self.subwords.push(e.copy());
        }
    }

    pub fn add_text(&mut self, s: &str) {
        self.text += &s.to_string();
        self.subwords.push(Box::new(UnquotedSubword::new(s)));
    }

    pub fn brace_expansion(&mut self) -> Vec<Word> {
        let mut ans = vec![Word::new()];

        for sub in self.subwords.iter_mut() {
            sub.brace_expansion(&mut ans);
        }

        ans
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Word> {
        if feeder.starts_with("#") {
            return None;
        }

        let left = core.word_nest.last().unwrap().to_string();
        let mut ans = Word::new();

        if feeder.starts_with("{}") {
            let sw = UnquotedSubword::new("{}");
            feeder.consume(2);
            ans.subwords.push(Box::new(sw));
            ans.text += &"{}";
        }else if feeder.starts_with("}") && left != "," {
            let sw = UnquotedSubword::new("}");
            feeder.consume(1);
            ans.subwords.push(Box::new(sw));
            ans.text += &"}";
        }

        loop {
            while let Some(sw) = subword::parse(feeder, core) {
                ans.text += &sw.get_text();
                ans.subwords.push(sw);
            }

            if feeder.len() == 0 {
                break;
            }else if feeder.starts_with("{"){
                let c = feeder.consume(1);
                ans.text += &c;
                let sw = UnquotedSubword::new(&c);
                ans.subwords.push(Box::new(sw));
                continue;
            }

            if left == "{" && feeder.starts_with("}") {
                break;
            }

            if left == "{" && feeder.starts_with(",") {
                break;
            }else if left == "," && ( feeder.starts_with(",") || feeder.starts_with("}") ) {
                break;
            }

            if feeder.starts_with(",") || feeder.starts_with("}") {
                let c = feeder.consume(1);
                ans.text += &c;
                let sw = UnquotedSubword::new(&c);
                ans.subwords.push(Box::new(sw));
            }else{
                break;
            }
        }

        if ans.text.len() == 0 {
            None
        }else{
            //dbg!("{:?}", &ans);
            Some(ans)
        }
    }
}
