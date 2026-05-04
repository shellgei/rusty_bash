//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod array;
pub mod subscript;
pub mod value;
pub mod variable;

use self::value::Value;
use self::variable::Variable;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct Substitution {
    pub text: String,
    pub left_hand: Variable,
    pub right_hand: Option<Value>,
    append: bool,
    lineno: usize,
    pub quoted: bool,
    pub reset_nameref: bool,
}

impl Substitution {
    pub fn eval(
        &mut self,
        core: &mut ShellCore,
        scope: Option<usize>,
        declare: bool,
    ) -> Result<(), ExecError> {
        core.db.set_param("LINENO", &self.lineno.to_string(), None)?;
        if self.right_hand.is_none() {
            return Ok(());
        }

        if core.db.exist_nameref(&self.left_hand.name) && ! self.reset_nameref {
            let mut circular_check_vec = vec![];
            let org_name = self.left_hand.name.clone();
            loop {
                self.left_hand.check_nameref(core)?;

                if circular_check_vec.is_empty() && org_name == self.left_hand.name {
                    break;
                }

                if circular_check_vec.contains(&self.left_hand.name) {
                    return Err(ExecError::CircularNameRef(org_name));
                }
                if ! core.db.exist_nameref(&self.left_hand.name) {
                    break;
                }
                circular_check_vec.push(self.left_hand.name.clone());
            }
        }

        let r = self.right_hand.as_mut().unwrap();
        r.eval(core, &self.left_hand.name, self.append)?;

        if r.is_obj() {
            return Ok(());
        }

        if declare && r.evaluated_array.is_some() {
            self.left_hand.index = None;
        }
        self.set_to_shell(core, scope)
    }

    pub fn reparse(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.left_hand.index.is_some() {
            self.left_hand
                .index
                .as_mut()
                .unwrap()
                .reparse(core, &self.left_hand.name)?;
        }

        if let Some(r) = self.right_hand.as_mut() {
            r.reparse(core, self.quoted)?;
        }
        Ok(())
    }

    pub fn localvar_inherit(&mut self, core: &mut ShellCore) {
        if self.right_hand.is_some() {
            return;
        }

        if let Some(d) = core.db.get_ref(&self.left_hand.name) {
            self.right_hand = Some(Value::from(d.clone()));
        }
    }

    fn restore_flag(core: &mut ShellCore, name: &str,
                    old_flags: &str, scope: usize) {
        for flag in old_flags.chars() {
            if flag == 'A' || flag == 'a' {
                continue;
            }
            if old_flags.contains(flag) {
                core.db.set_flag(&name, flag, scope);
            }
        }
    }

    fn set_whole_array(&mut self, core: &mut ShellCore, scope: usize) -> Result<(), ExecError> {
        let r = self.right_hand.as_mut().unwrap();
        if r.evaluated_array.is_none() {
            return Err(ExecError::Other("no array and no index".to_string()));
        }

        let a = r.evaluated_array.as_ref().unwrap();
        let name = &self.left_hand.name;
        let old_flags = core.db.get_flags(&name).to_string();

        if a.is_empty() && !self.append {
            if core.db.is_assoc(name) {
                core.db.init_assoc(name, Some(scope), true, false)?;
            } else {
                core.db
                    .init_array(name, Some(vec![]), Some(scope), false)?;
            }

            Self::restore_flag(core, name, &old_flags, scope);
            return Ok(());
        } else if !self.append {
            core.db.init(name, scope);
            Self::restore_flag(core, name, &old_flags, scope);
        }

        for e in a {
            match e.1 {
                //true if append
                false => core.db
                    .set_param2(name, &e.0, &e.2, Some(scope))?,
                true => core.db
                    .append_param2(name, &e.0, &e.2, Some(scope))?,
            }
        }
        Ok(())
    }

    fn set_array_elem(
        &mut self,
        core: &mut ShellCore,
        scope: usize,
        index: &str,
    ) -> Result<(), ExecError> {
        if index.is_empty() {
            return Err(ExecError::ArrayIndexInvalid(self.left_hand.text.clone()));
        }

        let r = self.right_hand.as_mut().unwrap();
        if let Some(v) = &r.evaluated_string {
            if self.append {
                return core
                    .db
                    .append_param2(&self.left_hand.name, index, v, Some(scope));
            } else {
                return core
                    .db
                    .set_param2(&self.left_hand.name, index, v, Some(scope));
            }
        }

        let msg = format!(
            "{}: cannot assign list to array member",
            &self.left_hand.text
        );
        Err(ExecError::Other(msg))
    }

    fn init_array(&mut self, core: &mut ShellCore, scope: usize) -> Result<(), ExecError> {
        let rhs_is_array = match self.right_hand.as_mut() {
            Some(r) => r.evaluated_array.is_some(),
            None => false,
        };

        match self.left_hand.get_index(core, rhs_is_array, self.append)? {
            Some(index) => self.set_array_elem(core, scope, &index),
            None => self.set_whole_array(core, scope),
        }
    }

    fn set_single(&mut self, core: &mut ShellCore, scope: usize) -> Result<(), ExecError> {
        let data = match self.right_hand.as_mut() {
            Some(r) => r.evaluated_string.clone().unwrap(),
            None => String::new(),
        };

        if self.append {
            core.db
                .append_param(&self.left_hand.name, &data, Some(scope))
        } else if self.reset_nameref {
            core.db.set_nameref(&self.left_hand.name, &data, Some(scope))
        } else {
            core.db.set_param(&self.left_hand.name, &data, Some(scope))
        }
    }

    fn set_to_shell(
        &mut self,
        core: &mut ShellCore,
        scope: Option<usize>,
    ) -> Result<(), ExecError> {
        let scope = core.db.get_target_scope(&self.left_hand.name, scope);
        let r = self.right_hand.as_mut().unwrap();

        if r.evaluated_string.is_some()
        && self.left_hand.index.is_none() {
            self.set_single(core, scope)
        } else {
            self.init_array(core, scope)
        }
    }

    fn eat_equal(&mut self, feeder: &mut Feeder) -> bool {
        if feeder.starts_with("+=") {
            self.append = true;
            self.text += &feeder.consume(2);
        } else if feeder.starts_with("=") {
            self.text += &feeder.consume(1);
        } else {
            return false;
        }

        true
    }

    fn eat_left_hand(
        &mut self,
        feeder: &mut Feeder,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        self.left_hand = match Variable::parse(feeder, core)? {
            Some(a) => a,
            None => return Ok(false),
        };
        self.text = self.left_hand.text.clone();
        self.lineno = self.left_hand.lineno;
        Ok(true)
    }

    pub fn parse(
        feeder: &mut Feeder,
        core: &mut ShellCore,
        permit_space: bool,
        permit_no_equal: bool,
    ) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();

        feeder.set_backup();
        match ans.eat_left_hand(feeder, core) {
            Ok(true) => {}
            Ok(false) => {
                feeder.rewind();
                return Ok(None);
            }
            Err(e) => {
                feeder.rewind();
                return Err(e);
            }
        }

        if !ans.eat_equal(feeder) {
            if permit_no_equal {
                feeder.pop_backup();
                return Ok(Some(ans));
            }
            feeder.rewind();
            return Ok(None);
        }
        feeder.pop_backup();

        if let Some(a) = Value::parse(feeder, core, permit_space)? {
            ans.text += &a.text.clone();
            ans.right_hand = Some(a);
        }

        Ok(Some(ans))
    }
}
