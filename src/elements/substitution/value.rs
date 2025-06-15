//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::word::WordMode;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use super::array::Array;
use crate::elements::word::Word;

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
    //pub evaluated_array: Option<HashMap<String, String>>,
    pub evaluated_array: Option<Vec<(String, String)>>,
}

impl Value {
    pub fn eval(&mut self, core: &mut ShellCore, name: &str, append: bool) -> Result<(), ExecError> {
        match self.value.clone() {
            ParsedDataType::Single(v) => self.eval_as_value(&v, core, name),
            ParsedDataType::Array(mut a) => self.eval_as_array(&mut a, core, name, append),
            ParsedDataType::None => {
                self.evaluated_string = Some("".to_string());
                Ok(())
            },
        }
    }

    fn eval_as_value(&mut self, w: &Word, core: &mut ShellCore, name: &str)
    -> Result<(), ExecError> {
        self.evaluated_string = match core.db.has_flag(&name, 'i') {
            true  => Some(w.eval_as_integer(core)?),
            false => Some(w.eval_as_value(core)?),
        };

        Ok(())
    }

    fn eval_as_array(&mut self, a: &mut Array, core: &mut ShellCore,
                     name: &str, append: bool) -> Result<(), ExecError> {
        let mut i = match append {
            false => 0,
            true  => core.db.index_based_len(&name) as isize,
        };

        let mut hash = vec![];
        let mut vec_assoc = vec![];
        let i_flag = core.db.has_flag(&name, 'i');
        let mut first = true;
        let mut assoc_no_index_mode = false;

        for (s, v) in a.eval(core, i_flag)? { 
            if assoc_no_index_mode {
                vec_assoc.push(v);
                continue;
            }

            if s.is_none() {
                if first && core.db.is_assoc(&name) {
                    assoc_no_index_mode = true;
                    vec_assoc.push(v);
                }else{
                    hash.push((i.to_string(), v));
                }
                i += 1;
                first = false;
                continue;
            }

            first = false;
            let index = match s.unwrap().eval(core, &name) {
                Ok(i) => i,
                Err(ExecError::ArithError(a,b))
                    => return Err(ExecError::ArithError(a,b)),
                Err(e) => {
                    e.print(core);
                    continue;
                },
            };

            if core.db.is_assoc(&name) {
                hash.push((index, v));
            }else{
                match index.parse::<isize>() {
                    Ok(j) => i = j,
                    Err(e) => {
                        eprintln!("{:?}", &e);
                        continue;
                    },
                }
                hash.push((index, v));
            }
            i += 1;
        }

        if assoc_no_index_mode {
            let mut key = String::new();
            for (i, d) in vec_assoc.iter().enumerate() {
                match i%2 {
                    0 => key = d.clone(),
                    _ => hash.push((key.clone(), d.clone())),
                }
            }

            if vec_assoc.len()%2 == 1 {
                hash.push((key.clone(), "".to_string()));
            }
        }

        self.evaluated_array = Some(hash);
        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();

        if let Some(a) = Array::parse(feeder, core)? {
            ans.text += &a.text;
            ans.value = ParsedDataType::Array(a);
        }else if let Ok(Some(mut w)) = Word::parse(feeder, core, None) {
            w.mode = Some(WordMode::RightOfSubstitution);
            ans.text += &w.text;
            ans.value = ParsedDataType::Single(w);
        }
        Ok(Some(ans))
    }
}
