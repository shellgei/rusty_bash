//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::array::Array;
use crate::core::database::data::Data;
use crate::elements::word::Word;
use crate::elements::word::WordMode;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub enum ParsedDataType {
    #[default]
    None,
    Single(Word),
    Array(Array),
    Obj(Box::<dyn Data>),
}

#[derive(Debug, Clone, Default)]
pub struct Value {
    pub text: String,
    pub value: ParsedDataType,
    pub evaluated_string: Option<String>,
    pub evaluated_array: Option<Vec<(String, bool, String)>>, //bool: true if append
}

impl From<Box::<dyn Data>> for Value {
    fn from(mut d: Box::<dyn Data>) -> Self {
        Self {
            text: (*d.get_print_string()).to_string(),
            value: ParsedDataType::Obj(d),
            ..Default::default()
        }
    }
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
            ParsedDataType::Obj(_) => {Ok(())},
            ParsedDataType::None => {
                self.evaluated_string = Some("".to_string());
                Ok(())
            }
        }
    }

    pub fn is_obj(&self) -> bool {
        match self.value {
            ParsedDataType::Obj(_) => true,
            _ => false,
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
        let mut i = match append {
            false => 0,
            true => core.db.index_based_len(name) as isize,
        };

        let mut hash = vec![];
        let mut vec_assoc = vec![];
        let i_flag = core.db.has_flag(name, 'i');
        let mut first = true;
        let mut assoc_no_index_mode = false;
        let assoc = core.db.is_assoc(name);

        for (pos, (s, append, v)) in a.eval(core, i_flag, assoc)?.into_iter().enumerate() {
            if assoc_no_index_mode {
                vec_assoc.push((v, append));
                continue;
            }

            if s.is_none() {
                if first && assoc {
                    assoc_no_index_mode = true;
                    vec_assoc.push((v, append));
                } else if assoc {
                    let msg = format!(
                        "{}: {}: must use subscript when assigning associative array",
                        &name, &a.words[pos].2.text
                    );
                    ExecError::Other(msg).print(core);
                } else {
                    hash.push((i.to_string(), append, v));
                }
                i += 1;
                first = false;
                continue;
            }

            first = false;
            let index = match s.unwrap().eval(core, name) {
                Ok(i) => i,
                Err(ExecError::ArithError(a, b)) => {
                    self.evaluated_array = Some(vec![]);
                    ExecError::ArithError(a, b).print(core);
                    return Ok(());
                }
                Err(e) => {
                    e.print(core);
                    continue;
                }
            };

            if assoc {
                hash.push((index, append, v));
            } else {
                match index.parse::<isize>() {
                    Ok(j) => i = j,
                    Err(e) => {
                        eprintln!("{:?}", &e);
                        continue;
                    }
                }
                hash.push((index, append, v));
            }
            i += 1;
        }

        if assoc_no_index_mode {
            let mut key = String::new();
            for (i, (d, append)) in vec_assoc.iter().enumerate() {
                match i % 2 {
                    0 => key = d.clone(),
                    _ => hash.push((key.clone(), *append, d.clone())),
                }
            }

            if vec_assoc.len() % 2 == 1 {
                hash.push((key.clone(), append, "".to_string()));
            }
        }

        self.evaluated_array = Some(hash);
        Ok(())
    }

    fn reparse_word(&mut self, w: &mut Word, core: &mut ShellCore) -> Result<(), ExecError> {
        let text = w.eval_as_value(core)?;
        let mut f = Feeder::new(&text.replace("~", "\\~"));
        if let Ok(Some(s)) = Self::parse(&mut f, core, true) {
            if !f.is_empty() {
                return Err(ExecError::InvalidName(text));
            }

            *self = s;
        }
        Ok(())
    }

    pub fn reparse(&mut self, core: &mut ShellCore, quoted: bool) -> Result<(), ExecError> {
        let v = self.value.clone();

        match v {
            ParsedDataType::Single(mut w) => self.reparse_word(&mut w, core),
            ParsedDataType::Array(a) => {
                if !quoted {
                    return Ok(());
                }
                let txt = "'".to_owned() + &a.text + "'";
                let mut w = Word::from(txt.as_str());
                self.reparse_word(&mut w, core)
            }
            _ => Ok(()),
        }
    }

    pub fn parse(
        feeder: &mut Feeder,
        core: &mut ShellCore,
        permit_space: bool,
    ) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();

        let wm = match permit_space {
            true => WordMode::PermitAnyChar,
            false => WordMode::Value,
        };

        if let Some(a) = Array::parse(feeder, core)? {
            ans.text += &a.text;
            ans.value = ParsedDataType::Array(a);
        } else if let Ok(Some(mut w)) = Word::parse(feeder, core, Some(wm)) {
            w.mode = Some(WordMode::RightOfSubstitution);
            ans.text += &w.text;
            ans.value = ParsedDataType::Single(w);
        }
        Ok(Some(ans))
    }
}
