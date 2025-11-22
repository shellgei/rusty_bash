//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod value;
mod variable;

use crate::{Feeder, ShellCore};
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use self::value::Value;
use self::variable::Variable;

#[derive(Debug, Clone, Default)]
pub struct Substitution {
    pub text: String,
    pub left_hand: Variable,
    right_hand: Option<Value>,
}

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore,
                layer: Option<usize>) -> Result<(), ExecError> {
        match self.right_hand.as_mut() {
            Some(r) => {
                r.eval(core)?;
                core.db.set_param(&self.left_hand.text,
                                  &r.evaluated_string, layer)
            },
            None => self.eval_no_right(core, layer),
        }
    }

    pub fn eval_no_right(&mut self, core: &mut ShellCore,
                         layer: Option<usize>) -> Result<(), ExecError> {
        let old_layer = core.db.get_layer_pos(&self.left_hand.text);
        let already_exit = old_layer.is_some() && old_layer == layer;

        if ! already_exit {
            core.db.set_param(&self.left_hand.text, "", layer)?;
        }
        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore,
        permit_space_in_value: bool,
        permit_no_equal: bool
    ) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        feeder.set_backup();

        match Variable::parse(feeder, core) {
            Ok(Some(v)) => {
                ans.text += &v.text.clone();
                ans.left_hand = v;
            },
            Ok(None) => { feeder.rewind(); return Ok(None); },
            Err(e)   => { feeder.rewind(); return Err(e); },
        }

        if ! feeder.starts_with("=") {
            if permit_no_equal {
                feeder.pop_backup();
                return Ok(Some(ans));
            }
            feeder.rewind();
            return Ok(None);
        }

        ans.text += &feeder.consume(1);
        feeder.pop_backup();

        if let Some(a) = Value::parse(feeder, core,
                             permit_space_in_value)? {
            ans.text += &a.text.clone();
            ans.right_hand = Some(a);
        }

        Ok(Some(ans))
    }
}
