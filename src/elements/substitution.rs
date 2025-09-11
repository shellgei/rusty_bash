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
    left_hand: Variable,
    right_hand: Value,
}

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore,
                layer: Option<usize>) -> Result<(), ExecError> {
        self.right_hand.eval(core)?;
        core.db.set_param(&self.left_hand.text,
                          &self.right_hand.evaluated_string, layer)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore,
                 permit_no_equal: bool) -> Result<Option<Self>, ParseError> {
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

        if let Some(a) = Value::parse(feeder, core)? {
            ans.text += &a.text.clone();
            ans.right_hand = a;
        }

        //dbg!("{:?}", &ans);
        Ok(Some(ans))
    }
}
