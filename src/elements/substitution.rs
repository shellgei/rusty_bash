//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod array;
pub mod subscript;
pub mod variable;
pub mod value;

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use self::value::Value;
use self::variable::Variable;

#[derive(Debug, Clone, Default)]
pub struct Substitution {
    pub text: String,
    pub left_hand: Variable,
    right_hand: Value,
    append: bool,
    lineno: usize,
}

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore, layer: Option<usize>) -> Result<(), ExecError> {
        core.db.set_param("LINENO", &self.lineno.to_string(), None)?;
        if let Err(e) = self.right_hand.eval(core, &self.left_hand.name, self.append) {
            core.db.exit_status = 1;
            return Err(e);
        }

        let ans = self.set_to_shell(core, layer);
        if ! ans.is_ok() {
            core.db.exit_status = 1;
        }
        ans
    }

    pub fn get_string_for_eval(&self, core: &mut ShellCore) -> Result<String, ExecError> {
        let mut splits = self.text.split("=");
        let front = splits.nth(0).unwrap().to_owned() + "=";
        let rear = self.right_hand.value.get_evaluated_text(core)?;

        Ok(front + &rear)
    }

    fn set_array(&mut self, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
        match self.get_index(core)? {
            None => {
                if let Some(a) = &self.right_hand.evaluated_array {
                    if a.is_empty() {
                        core.db.set_array(&self.left_hand.name, vec![], Some(layer))?;
                        return Ok(());
                    }

                    core.db.init(&self.left_hand.name, layer);
                    for e in a {
                        core.db.set_param2(&self.left_hand.name, &e.0, &e.1, Some(layer))?;
                    }
                    return Ok(());
                }
                return Err(ExecError::Other("no array and no index".to_string()));
            },
            Some(index) => {
                if index.is_empty() {
                    return Err(ExecError::Other(format!("{}[]: invalid index", &self.left_hand.name)));
                }
                if let Some(v) = &self.right_hand.evaluated_string {
                    return core.db.set_param2(&self.left_hand.name, &index, &v, Some(layer));
                }
                return Err(ExecError::Other("indexed to non array variable".to_string()));
            },
        }
    }

    fn set_to_shell(&mut self, core: &mut ShellCore, layer: Option<usize>)
    -> Result<(), ExecError> {
        let layer = core.db.get_target_layer(&self.left_hand.name, layer);

        if self.right_hand.evaluated_string.is_some() && self.left_hand.index.is_none() {
            let data = self.right_hand.evaluated_string.clone().unwrap();
            if self.append {
                return core.db.append_param(&self.left_hand.name, &data, Some(layer));
            }else{
                return core.db.set_param(&self.left_hand.name, &data, Some(layer));
            }
        }

        self.set_array(core, layer)
    }

    pub fn get_index(&mut self, core: &mut ShellCore) -> Result<Option<String>, ExecError> {
        match self.left_hand.get_index(core)? {
            Some(s) => return Ok(Some(s)),
            None => {
                if core.db.is_array(&self.left_hand.name)
                    && ! self.append
                    && self.right_hand.evaluated_array.is_none() {
                    return Ok(Some("0".to_string()));
                }
            },
        }
        Ok(None)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        feeder.set_backup();

        let mut ans = Self::default();

        ans.left_hand = match Variable::parse(feeder, core)? {
            Some(a) => a,
            None => return Ok(None),
        };
        ans.text = ans.left_hand.text.clone();
        ans.lineno = ans.left_hand.lineno;

        if feeder.starts_with("+=") {
            ans.append = true;
            ans.text += &feeder.consume(2);
        }else if feeder.starts_with("=") {
            ans.text += &feeder.consume(1);
        }else {
            feeder.rewind();
            return Ok(None);
        }
        feeder.pop_backup();

        if let Some(a) = Value::parse(feeder, core)? {
            ans.text += &a.text.clone();
            ans.right_hand = a;
        }

        Ok(Some(ans))
    }
}
