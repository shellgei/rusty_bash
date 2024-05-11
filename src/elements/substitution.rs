//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::array::Array;
use crate::elements::word::Word;

#[derive(Debug, Clone)]
pub struct Substitution {
    pub text: String,
    pub key: String,
    value: Option<Word>,
    array: Option<Array>,
}

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore) -> (Option<String>, Option<Vec<String>>) {
        match (self.value.clone(), self.array.clone()) {
            (None, None) => return (Some("".to_string()), None),
            (Some(v), None) => {
                match Self::eval_value(&v, core) {
                    Some(s) => return (Some(s), None),
                    None => return (None, None),
                }
            },
            (None, Some(mut a)) => {
                match a.eval(core) {
                    Some(values) => return (None, Some(values)),
                    None => return (None, None),
                }
            },
            _ => return (None, None), 
        }
    }

    fn eval_value(s: &Word, core: &mut ShellCore) -> Option<String> {
         s.eval_as_value(core)
    }

    pub fn new() -> Substitution {
        Substitution {
            text: String::new(),
            key: String::new(),
            value: None,
            array: None,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        let len = feeder.scanner_name_and_equal(core);
        if len == 0 {
            return None;
        }

        let mut ans = Self::new();

        let mut name_eq = feeder.consume(len);
        ans.text += &name_eq;
        name_eq.pop();
        ans.key = name_eq.clone();

        if let Some(a) = Array::parse(feeder, core) {
            ans.text += &a.text;
            ans.array = Some(a);
            Some(ans)
        }else if let Some(w) = Word::parse(feeder, core) {
            ans.text += &w.text;
            ans.value = Some(w);
            Some(ans)
        }else {
            None
        }
    }
}
