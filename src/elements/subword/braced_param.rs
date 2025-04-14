//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod optional_operation;
mod parse;

use crate::{Feeder, ShellCore};
use crate::elements::subword;
use crate::elements::subword::Subword;
use crate::elements::subword::simple::SimpleSubword;
use crate::elements::subscript::Subscript;
use crate::utils;
use crate::error::exec::ExecError;
use self::optional_operation::OptionalOperation;

#[derive(Debug, Clone, Default)]
pub struct Param {
    name: String,
    subscript: Option<Subscript>,
}

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    text: String,
    array: Option<Vec<String>>,
    param: Param,
    optional_operation: Option<Box<dyn OptionalOperation>>,
    unknown: String,
    is_array: bool,
    num: bool,
    indirect: bool,
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str { &self.text.as_ref() }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<Vec<Box<dyn Subword>>, ExecError> {
        self.check()?;

        if self.indirect && self.has_aster_or_atmark_subscript() { // ${!name[@]}, ${!name[*]}
            self.index_replace(core)?;
            return Ok(vec![]);
        }

        if self.indirect {
            self.indirect_replace(core)?;
            self.check()?;
        }

        if self.has_aster_or_atmark_subscript()
        || self.param.name == "@" 
        || self.param.name == "*" {
            if let Some(s) = self.optional_operation.as_mut() {
                if s.is_substr() {
                    let mut arr = vec![];
                    s.set_array(&self.param, &mut arr, &mut self.text, core)?;
                    self.array = Some(arr);
                    return Ok(vec![]);
                }
            }
        }

        if self.param.name == "*" {
            self.array = Some(core.db.get_position_params());
        }

        match self.param.subscript.is_some() {
            true  => self.subscript_operation(core)?,
            false => self.non_subscript_operation(core)?,
        }
        self.ans()
    }

    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }

    fn is_array(&self) -> bool {self.is_array && ! self.num}
    fn get_array_elem(&self) -> Vec<String> {self.array.clone().unwrap_or_default()}

    fn split(&self, ifs: &str, prev_char: Option<char>) -> Vec<(Box<dyn Subword>, bool)>{ 
        if (self.param.name != "@" && self.param.name != "*")
        || ifs.starts_with(" ") || self.array.is_none() {
            let f = |s| Box::new( SimpleSubword {text: s}) as Box<dyn Subword>;

            let ans = subword::split(&self.boxed_clone(), ifs, prev_char);
            return ans.iter().map(|s| (f(s.0.to_string()), s.1)).collect();
        }

        let mut ans = vec![];
        let mut tmp = SimpleSubword{ text: "".to_string() };
        for p in self.array.clone().unwrap() {
            tmp.text = p.clone();
            ans.push((tmp.boxed_clone(), true));
        }
        ans
    }
}

impl BracedParam {
    fn ans(&mut self) -> Result<Vec<Box<dyn Subword>>, ExecError> {
        match self.optional_operation.as_mut() {
            Some(op) => Ok(op.get_alternative()),
            None     => Ok(vec![]),
        }
    }

    fn has_aster_or_atmark_subscript(&self) -> bool {
        if self.param.subscript.is_none() {
            return false;
        }
        let sub = &self.param.subscript.as_ref().unwrap().text;
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

        if self.param.subscript.is_some() {
            if self.param.name == "@" || self.param.name == "*" {
                return Err(ExecError::BadSubstitution(self.param.name.clone()));
            }
        }
        Ok(())
    }

    fn index_replace(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.optional_operation.is_some() {
            let msg = core.db.get_array_all(&self.param.name).join(" ");
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
        sw.is_array = false;
        sw.num = false;

        sw.substitute(core)?;

        if sw.text.contains('[') {
            let mut feeder = Feeder::new(&("${".to_owned() + &sw.text + "}" ));
            if let Ok(Some(mut bp)) = BracedParam::parse(&mut feeder, core) {
                bp.substitute(core)?;
                self.param.name = bp.param.name;
                self.param.subscript = bp.param.subscript;
            }else{
                return Err(ExecError::InvalidName(sw.text.clone()));
            }
        }else{
            self.param.name = sw.text.clone();
            self.param.subscript = None;
        }

        if ! utils::is_param(&self.param.name) {
            return Err(ExecError::InvalidName(self.param.name.clone()));
        }
        Ok(())
    }

    fn non_subscript_operation(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
            let value = core.db.get_param(&self.param.name).unwrap_or_default();
            self.text = match self.num {
                true  => value.chars().count().to_string(),
                false => value.to_string(),
            };
    
            self.text = self.optional_operation(self.text.clone(), core)?;
            Ok(())
    }

    fn subscript_operation(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let index = self.param.subscript.clone().unwrap().eval(core, &self.param.name)?;

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

        let arr = core.db.get_array_all(&self.param.name);
        if self.num && (index.as_str() == "@" || index.as_str() == "*" ) {
            self.text = arr.len().to_string();
            self.array = Some(arr);
            return Ok(());
        }
        if index.as_str() == "@" {
            self.atmark_operation(core)
        }else{
            self.text = core.db.get_array_elem(&self.param.name, &index).unwrap();
            if self.num {
                self.text = self.text.chars().count().to_string();
            }
            self.text = self.optional_operation(self.text.clone(), core)?;
            Ok(())
        }
    }

    fn atmark_operation(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut arr = core.db.get_array_all(&self.param.name);
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
