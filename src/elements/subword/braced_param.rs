//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod parse;
mod indirect;
mod subscript;

use crate::elements::braced_param_ext::BracedExcludeension;
use crate::elements::substitution::variable::Variable;
use crate::elements::subword::Subword;
use crate::error::exec::ExecError;
use crate::{ShellCore, utils};
use crate::utils::splitter;

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    text: String,
    array: Option<Vec<String>>,
    param: Variable,
    extension: Option<Box<dyn BracedExcludeension>>,
    unknown: String,
    treat_as_array: bool,
    num: bool,
    indirect: bool,
}

impl From<&str> for BracedParam {
    fn from(s: &str) -> Self {
        let mut ans: Self = Default::default();
        ans.text = "$".to_owned() + s;
        ans.param.text = s.to_string();
        ans.param.name = s.to_string();
        ans
    }
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str {
        self.text.as_ref()
    }
    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if core.db.exist_nameref(&self.param.name) && ! self.indirect {
            self.param.solve_nameref(core)?;
            return self.substitute(core);
        }
        self.check()?;

        if self.indirect && ! self.indirect_preparation(core)? {
            return Ok(());
        }

        if self.param.is_array() 
        && let Some(op) = self.extension.as_mut()
        && op.has_array_replace() {
            return self.array_replace(core);
        }

        match self.param.index.is_some() {
            true => self.subscript_operation(core),
            false => self.non_subscript_operation(core),
        }
    }

    fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    fn is_array(&self) -> bool {
        self.treat_as_array
    }

    fn get_elem(&mut self) -> Vec<String> {
        if let Some(op) = self.extension.as_mut()
        && op.array_to_single() {
            return vec![self.text.clone()];
        }

        self.array.clone().unwrap_or_default()
    }

    fn alter(&mut self) -> Result<Vec<Box<dyn Subword>>, ExecError> {
        match self.extension.as_mut() {
            Some(op) => Ok(op.get_alternative()),
            None => Ok(vec![]),
        }
    }

    fn split(&self, ifs: &str, strip_left: bool) -> Option<Vec<(Box<dyn Subword>, bool)>> {
        if self.text.is_empty() {
            return None;
        }

        let asterisk = self.param.index.is_some() 
                       && self.param.index.as_ref().unwrap().text == "[*]"
                       || self.param.name == "*";

        if ifs.is_empty() && asterisk {
            return self.make_split();
        }

        if (!self.treat_as_array && !asterisk)
            || ifs.starts_with(" ")
            || self.array.is_none()
        {
            let splits = splitter::split(&self.text, ifs, strip_left);
            if splits.is_none() {
                return None;
            }

            return Some(splits.unwrap()
                .iter()
                .map(|s| (From::from(&s.0), s.1))
                .collect());
        }

        self.make_split()
    }

    fn set_heredoc_flag(&mut self) {
        self.extension
            .iter_mut()
            .for_each(|e| e.set_heredoc_flag());
    }
}

impl BracedParam {
    fn check(&mut self) -> Result<(), ExecError> {
        if self.param.name.is_empty() || !utils::is_param(&self.param.name) {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }
        if !self.unknown.is_empty() && !self.unknown.starts_with(",") {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }

        if self.param.index.is_some() && self.param.is_pos_param_array() {
            return Err(ExecError::BadSubstitution(self.param.name.clone()));
        }
        Ok(())
    }

    fn make_split(&self) -> Option<Vec<(Box<dyn Subword>, bool)>> {
        if self.array.is_none() {
            return None;
        }

        let mut ans = vec![];
        for p in self.array.clone().unwrap() {
            ans.push((From::from(&p), true));
        }
        Some(ans)
    }

    fn index_replace(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.extension.is_some() {
            let msg = core.db.get_vec(&self.param.name, true)?.join(" ");
            return Err(ExecError::InvalidName(msg));
        }

        if !core.db.exist(&self.param.name) {
            self.text = "".to_string();
            return Ok(());
        }

        if !core.db.is_array(&self.param.name) && !core.db.is_assoc(&self.param.name) {
            self.text = "0".to_string();
            return Ok(());
        }

        let arr = core.db.get_indexes_all(&self.param.name);
        self.array = Some(arr.clone());
        self.text = arr.join(" ");

        Ok(())
    }

    fn array_replace(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut arr = vec![];
        let op = self.extension.as_mut().unwrap();
        op.init_array(&self.param, &mut arr, &mut self.text, core)?;
        self.array = Some(arr.clone());
        if let Some(index) = &self.param.index {
            if index.text == "[*]" {
                self.text = arr.join(&core.db.get_ifs_head());
            }else if index.text == "[@]" {
                self.text = arr.join(" ");
            }
        }

        Ok(())
    }

    fn non_subscript_operation(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.param.name == "*" || self.param.name == "@" {
            self.array = Some(core.db.get_position_params());
        }

        let value = core.db.get_param(&self.param.name).unwrap_or_default();
        self.text = match self.num {
            true => core.db.get_braced_param_hash_length(&self.param.name)?.to_string(),
            false => value.to_string(),
        };

        if let Some(op) = self.extension.as_mut() {
            self.text = op.exec(&self.param, &self.text, core)?;
        }

        //self.text = self.extension(self.text.clone(), core)?;
        Ok(())
    }
}
