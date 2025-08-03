//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::array::Array;
use crate::elements::word::Word;
use crate::elements::word::WordMode;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub enum ParsedDataType {
    #[default]
    None,
    Single(Word),
    Array(Array),
}

#[derive(Debug, Clone, Default)]
pub struct Value {
    pub text: String,
    pub value: ParsedDataType,
    pub evaluated_string: Option<String>,
    pub evaluated_array: Option<HashMap<String, String>>,
}

impl Value {
    pub fn eval(
        &mut self,
        core: &mut ShellCore,
        name: &str,
        append: bool,
    ) -> Result<(), ExecError> {
        match self.value.clone() {
            ParsedDataType::Single(v) => self.eval_as_value(&v, core, name),
            ParsedDataType::Array(mut a) => self.eval_as_array(&mut a, core, name, append),
            ParsedDataType::None => {
                self.evaluated_string = Some("".to_string());
                Ok(())
            }
        }
    }

    fn eval_as_value(
        &mut self,
        w: &Word,
        core: &mut ShellCore,
        name: &str,
    ) -> Result<(), ExecError> {
        self.evaluated_string = match core.db.has_flag(name, 'i') {
            true => Some(w.eval_as_integer(core)?),
            false => Some(w.eval_as_value(core)?),
        };

        Ok(())
    }

    fn eval_as_array(
        &mut self,
        a: &mut Array,
        core: &mut ShellCore,
        name: &str,
        append: bool,
    ) -> Result<(), ExecError> {
        let prev = match append {
            true => core.db.get_vec(name, true)?,
            false => vec![],
        };

        let mut i = 0;
        let mut hash = HashMap::new();
        for e in prev {
            hash.insert(i.to_string(), e);
            i += 1;
        }

        let i_flag = core.db.has_flag(name, 'i');
        let values = a.eval(core, i_flag)?;

        for (s, v) in values {
            match s {
                Some(mut sub) => {
                    let index = match sub.eval(core, name) {
                        Ok(i) => i,
                        Err(e) => {
                            e.print(core);
                            continue;
                        }
                    };
                    if core.db.is_assoc(name) {
                        hash.insert(index, v);
                    } else {
                        match index.parse::<usize>() {
                            Ok(j) => i = j,
                            Err(e) => {
                                eprintln!("{:?}", &e);
                                continue;
                            }
                        }
                        hash.insert(index, v);
                    }
                }
                None => {
                    hash.insert(i.to_string(), v);
                }
            }
            i += 1;
        }
        self.evaluated_array = Some(hash);
        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();

        if let Some(a) = Array::parse(feeder, core)? {
            ans.text += &a.text;
            ans.value = ParsedDataType::Array(a);
        } else if let Ok(Some(mut w)) = Word::parse(feeder, core, None) {
            w.mode = Some(WordMode::RightOfSubstitution);
            ans.text += &w.text;
            ans.value = ParsedDataType::Single(w);
        }
        Ok(Some(ans))
    }
}
