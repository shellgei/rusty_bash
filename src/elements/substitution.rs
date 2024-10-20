//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::core::data::Value;
use super::array::Array;
use super::word::Word;

#[derive(Debug, Clone)]
pub struct Substitution {
    pub text: String,
    pub key: String,
    pub value: Value,
    pub append: bool,
}

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore) -> Value {
        match self.value.clone() {
            Value::None      => Value::EvaluatedSingle("".to_string()),
            Value::Single(v) => self.eval_as_value(&v, core),
            Value::Array(a)  => self.eval_as_array(&mut a.clone(), core),
            _                => Value::None,
        }
    }

    fn eval_as_value(&self, w: &Word, core: &mut ShellCore) -> Value {
        let prev = match self.append {
            true  => core.data.get_param(&self.key),
            false => "".to_string(),
        };

        match w.eval_as_value(core) {
            Some(s) => Value::EvaluatedSingle(prev + &s),
            None    => Value::None,
        }
    }

    fn eval_as_array(&self, a: &mut Array, core: &mut ShellCore) -> Value {
        let prev = match self.append {
            true  => core.data.get_array_all(&self.key),
            false => vec![],
        };

        match a.eval(core) {
            Some(values) => Value::EvaluatedArray([prev, values].concat()),
            None         => Value::None,
        }
    }

    pub fn new() -> Substitution {
        Substitution {
            text: String::new(),
            key: String::new(),
            value: Value::None,
            append: false,
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
        if name_eq.ends_with("+") {
            ans.append = true;
            name_eq.pop();
        }

        ans.key = name_eq.clone();

        if let Some(a) = Array::parse(feeder, core) {
            ans.text += &a.text;
            ans.value = Value::Array(a);
            Some(ans)
        }else if let Some(w) = Word::parse(feeder, core, false) {
            ans.text += &w.text;
            ans.value = Value::Single(w);
            Some(ans)
        }else {
            Some(ans)
        }
    }
}
