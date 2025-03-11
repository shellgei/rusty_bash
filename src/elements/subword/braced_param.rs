//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod optional_operation;
mod value_check;
mod substr;
mod remove;
mod parse;
mod case_conv;

use crate::{Feeder, ShellCore};
use crate::elements::subword::Subword;
use crate::elements::subscript::Subscript;
use crate::elements::word::Word;
use crate::utils;
use crate::error::exec::ExecError;
use self::optional_operation::OptionalOperation;
use self::remove::Remove;

#[derive(Debug, Clone, Default)]
pub struct Param {
    name: String,
    subscript: Option<Subscript>,
}

/*
trait OptionalOperation {
    fn exec(&mut self, _: &Param, _: &String, _: &mut ShellCore) -> Result<String, ExecError>;
    fn boxed_clone(&self) -> Box<dyn OptionalOperation>;
    fn get_text(&self) -> String {"".to_string()}
    fn is_substr(&self) -> bool {false}
    fn is_value_check(&self) -> bool {false}
    fn set_array(&mut self, _: &Param, _: &mut Vec<String>,
                 _: &mut String, _: &mut ShellCore) -> Result<(), ExecError> {
        Ok(())
    }
    fn get_alternative(&self) -> Vec<Box<dyn Subword>> { vec![] }
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Box<dyn OptionalOperation>>, ParseError> {
    if let Some(a) = Replace::parse(feeder, core)?{ Ok(Some(Box::new(a))) }
    else{ Ok(None) }
}

impl Clone for Box::<dyn OptionalOperation> {
    fn clone(&self) -> Box<dyn OptionalOperation> {
        self.boxed_clone()
    }
}

impl Debug for dyn OptionalOperation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}
*/

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    text: String,
    array: Vec<String>,
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
                    s.set_array(&self.param, &mut self.array, &mut self.text, core)?;
                    return Ok(vec![]);
                }
            }
        }

        match self.param.subscript.is_some() {
            true  => self.subscript_operation(core)?,
            false => self.non_subscript_operation(core)?,
        }
        self.ans()
    }

    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }

    fn is_array(&self) -> bool {self.is_array && ! self.num}
    fn get_array_elem(&self) -> Vec<String> {self.array.clone()}
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

        self.array = core.db.get_indexes_all(&self.param.name);
        self.text = self.array.join(" ");

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
        if ! core.db.is_array(&self.param.name) && ! core.db.is_assoc(&self.param.name) {
            self.text = "".to_string();
            return Ok(());
        }

        let index = self.param.subscript.clone().unwrap().eval(core, &self.param.name)?;

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
        self.array = core.db.get_array_all(&self.param.name);
        if self.num {
            self.text = self.array.len().to_string();
            return Ok(());
        }

        self.text = match self.num {
            true  => core.db.len(&self.param.name).to_string(),
            false => core.db.get_array_elem(&self.param.name, "@").unwrap(),
        };

        if self.array.len() <= 1 || self.has_value_check() {
            self.text = self.optional_operation(self.text.clone(), core)?;
        }else {
            for i in 0..self.array.len() {
                self.array[i] = self.optional_operation(self.array[i].clone(), core)?;
            }
            self.text = self.array.join(" ");
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
