//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod optional_operation;
mod parse;

use crate::{Feeder, ShellCore};
use crate::elements::subword::Subword;
use crate::elements::substitution::subscript::Subscript;
use crate::elements::substitution::variable::Variable;
use crate::utils;
use crate::utils::splitter;
use crate::error::exec::ExecError;
use self::optional_operation::OptionalOperation;

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    text: String,
    array: Option<Vec<String>>,
    param: Variable,
    optional_operation: Option<Box<dyn OptionalOperation>>,
    unknown: String,
    treat_as_array: bool,
    num: bool,
    indirect: bool,
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str { &self.text.as_ref() }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        self.check()?;

        if self.indirect && self.has_aster_or_atmark_subscript() { // ${!name[@]}, ${!name[*]}
            self.index_replace(core)?;
            return Ok(());
        }

        if self.indirect {
            self.indirect_replace(core)?;
            self.check()?;
        }

        if self.has_aster_or_atmark_subscript()
        || self.param.name == "@" 
        || self.param.name == "*" {
            if let Some(s) = self.optional_operation.as_mut() {
                if s.has_array_replace() {
                    let mut arr = vec![];
                    s.set_array(&self.param, &mut arr, &mut self.text, core)?;
                    self.array = Some(arr);
                    return Ok(());
                }
            }
        }

        match self.param.index.is_some() {
            true  => self.subscript_operation(core),
            false => self.non_subscript_operation(core),
        }
    }

    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }

    fn is_array(&self) -> bool {self.treat_as_array }
    fn get_array_elem(&self) -> Vec<String> {self.array.clone().unwrap_or_default()}

    fn alter(&mut self) -> Result<Vec<Box<dyn Subword>>, ExecError> {
        match self.optional_operation.as_mut() {
            Some(op) => Ok(op.get_alternative()),
            None     => Ok(vec![]),
        }
    }

    fn split(&self, ifs: &str, prev_char: Option<char>) -> Vec<(Box<dyn Subword>, bool)>{ 
        if self.text == "" {
            return vec![];
        }

        if (self.param.name != "@" && self.param.name != "*")
        || ifs.starts_with(" ") || self.array.is_none() {
            return splitter::split(&self.get_text(), ifs, prev_char).iter()
                .map(|s| ( From::from(&s.0), s.1)).collect();
        }

        let mut ans = vec![];
        for p in self.array.clone().unwrap() {
            ans.push( (From::from(&p), true) );
        }
        ans
    }

    fn set_heredoc_flag(&mut self) {
        self.optional_operation.iter_mut()
            .for_each(|e| e.set_heredoc_flag());
    }
}

impl BracedParam {
    fn has_aster_or_atmark_subscript(&self) -> bool {
        if self.param.index.is_none() {
            return false;
        }
        let sub = &self.param.index.as_ref().unwrap().text;
        sub == "[*]" || sub == "[@]"
    }

    fn check(&mut self) -> Result<(), ExecError> {
        if self.param.name.is_empty() || ! utils::is_param(&self.param.name) {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }
        if self.unknown.len() > 0 
        && ! self.unknown.starts_with(",") {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }

        if self.param.index.is_some() {
            if self.param.name == "@" || self.param.name == "*" {
                return Err(ExecError::BadSubstitution(self.param.name.clone()));
            }
        }
        Ok(())
    }

    fn index_replace(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.optional_operation.is_some() {
            let msg = core.db.get_array_all(&self.param.name, true).join(" ");
            return Err(ExecError::InvalidName(msg));
        }

        if ! core.db.has_value(&self.param.name) {
            self.text = "".to_string();
            return Ok(());
        }

        if ! core.db.is_array(&self.param.name) && ! core.db.is_assoc(&self.param.name) {
            self.text = "0".to_string();
            return Ok(());
        }

        let arr = core.db.get_indexes_all(&self.param.name);
        self.array = Some(arr.clone());
        self.text = arr.join(" ");

        Ok(())
    }

    fn indirect_replace(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut sw = self.clone();
        sw.indirect = false;
        sw.unknown = String::new();
        sw.treat_as_array = false;
        sw.num = false;

        sw.substitute(core)?;

        if sw.text.contains('[') {
            let mut feeder = Feeder::new(&("${".to_owned() + &sw.text + "}" ));
            if let Ok(Some(mut bp)) = BracedParam::parse(&mut feeder, core) {
                bp.substitute(core)?;
                self.param.name = bp.param.name;
                self.param.index = bp.param.index;
            }else{
                return Err(ExecError::InvalidName(sw.text.clone()));
            }
        }else{
            self.param.name = sw.text.clone();
            self.param.index = None;
        }

        if ! utils::is_param(&self.param.name) {
            return Err(ExecError::InvalidName(self.param.name.clone()));
        }
        Ok(())
    }

    fn non_subscript_operation(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
            if self.param.name == "*" || self.param.name == "@" {
                self.array = Some(core.db.get_position_params());
            }

            let value = core.db.get_param(&self.param.name).unwrap_or_default();
            self.text = match self.num {
                true  => core.db.get_len(&self.param.name)?.to_string(),//value.chars().count().to_string(),
                false => value.to_string(),
            };
    
            self.text = self.optional_operation(self.text.clone(), core)?;
            Ok(())
    }

    fn subscript_operation(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let index = self.param.index.clone().unwrap().eval(core, &self.param.name)?;

        if self.num {
            self.text = core.db.get_elem_len(&self.param.name, &index)?.to_string();
            return Ok(());
        }

        if core.db.has_value(&self.param.name)
        && ! core.db.is_array(&self.param.name)
        && ! core.db.is_assoc(&self.param.name) {
            let param = core.db.get_param(&self.param.name);
            self.text = match index.as_str() { //case: a=aaa; echo ${a[@]}; (output: aaa)
                "@" | "*" | "0" => param.unwrap_or("".to_string()),
                 _ => "".to_string(),
            };
            return Ok(());
        }

        if index.as_str() == "@" {
            self.atmark_operation(core)
        }else{
            let tmp = core.db.get_array_elem(&self.param.name, &index).unwrap();
            self.text = self.optional_operation(tmp, core)?;
            Ok(())
        }
    }

    fn atmark_operation(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut arr = core.db.get_array_all(&self.param.name, true);
        self.array = Some(arr.clone());
        if self.num {
            self.text = arr.len().to_string();
            return Ok(());
        }

        self.text = match self.num {
            true  => core.db.len(&self.param.name).to_string(),
            false => core.db.get_array_elem(&self.param.name, "@").unwrap(),
        };

        if arr.len() <= 1 || self.has_value_check() {
            self.text = self.optional_operation(self.text.clone(), core)?;
        }else {
            for i in 0..arr.len() {
                arr[i] = self.optional_operation(arr[i].clone(), core)?;
            }
            self.text = arr.join(" ");
            self.array = Some(arr);
        }
        Ok(())
    }
    
    fn has_value_check(&mut self) -> bool {
        match self.optional_operation.as_mut() {
            Some(op) => op.is_value_check(),
            _ => false,
        }
    }

    fn optional_operation(&mut self, text: String, core: &mut ShellCore) -> Result<String, ExecError> {
        match self.optional_operation.as_mut() {
            Some(op) => op.exec(&self.param, &text, core),
            None => Ok(text.clone()),
        }
    }
}
