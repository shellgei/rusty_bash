//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod value_check;
mod substr;
mod remove;
mod replace;
mod parse;
mod case_conv;

use crate::{Feeder, ShellCore};
use crate::elements::subword::Subword;
use crate::elements::subscript::Subscript;
use crate::elements::word::Word;
use crate::utils;
use crate::error::exec::ExecError;
use self::case_conv::CaseConv;
use self::remove::Remove;
use self::replace::Replace;
use self::value_check::ValueCheck;
use core::fmt;
use core::fmt::Debug;

#[derive(Debug, Clone, Default)]
struct Param {
    name: String,
    subscript: Option<Subscript>,
}

trait OptionalOperation {
    fn exec(&mut self, _: &Param, _: &String, _: &mut ShellCore) -> Result<String, ExecError>;
    fn boxed_clone(&self) -> Box<dyn OptionalOperation>;
    fn get_text(&self) -> String {"".to_string()}
    fn is_substr(&self) -> bool {false}
    fn set_array(&mut self, _: &Param, _: &mut Vec<String>,
                    _: &mut String, _: &mut ShellCore) -> Result<(), ExecError> {
        Ok(())
    }

    fn set_partial_position_params(&mut self, _: &mut Vec<String>,
                    _: &mut String, _: &mut ShellCore) -> Result<(), ExecError> {
        Ok(())
    }

    fn set_partial_array(&mut self, _: &str, _: &mut Vec<String>,
                    _: &mut String, _: &mut ShellCore) -> Result<(), ExecError> {
        Ok(())
    }
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

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    text: String,
    array: Vec<String>,

    param: Param,

    optional_operation: Option<Box<dyn OptionalOperation>>,

    //replace: Option<Replace>,
    //substr: Option<Substr>,
    remove: Option<Remove>,
    value_check: Option<ValueCheck>,
    case_conv: Option<CaseConv>,

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

        if self.indirect {
            if let Some(sub) = &self.param.subscript {
                if sub.text == "[*]" || sub.text == "[@]" {
                    if self.optional_operation.is_some() 
                    //|| self.substr.is_some()
                    || self.remove.is_some()
                    || self.value_check.is_some() {
                        let msg = core.db.get_array_all(&self.param.name).join(" ");
                        return Err(ExecError::InvalidName(msg));
                    }

                    self.index_replace(core)?;
                    return self.ans();
                }
            }
            self.indirect_replace(core)?;
        }

        if let Some(sub) = &self.param.subscript {
            if sub.text == "[*]" || sub.text == "[@]" {
                if let Some(s) = self.optional_operation.as_mut() {
                    if s.is_substr() {
                        //s.set_partial_array(&self.param.name, &mut self.array, &mut self.text, core)?;
                        s.set_array(&self.param, &mut self.array, &mut self.text, core)?;
                        return self.ans();
                    }
                }
            }
        }

        if self.param.subscript.is_some() {
            if self.param.name == "@" {
                return Err(ExecError::BadSubstitution("@".to_string()));
            }
            self.subscript_operation(core)?;
            return self.ans();
        }

        if self.param.name == "@" {
            if let Some(s) = self.optional_operation.as_mut() {
                if s.is_substr() {
                   // s.set_partial_position_params(&mut self.array, &mut self.text, core)?;
                    s.set_array(&self.param, &mut self.array, &mut self.text, core)?;
                    return self.ans();
                }
            }
        }

        let value = core.db.get_param(&self.param.name).unwrap_or_default();
        self.text = match self.num {
            true  => value.chars().count().to_string(),
            false => value.to_string(),
        };

        self.text = self.optional_operation(self.text.clone(), core)?;
        self.ans()
    }

    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }

    fn is_array(&self) -> bool {self.is_array && ! self.num}
    fn get_array_elem(&self) -> Vec<String> {self.array.clone()}
}

impl BracedParam {
    fn get_alternative_subwords(&self) -> Vec<Box<dyn Subword>> {
        if self.value_check.is_none() {
            return vec![];
        }

        let check = self.value_check.clone().unwrap();
        match &check.alternative_value {
            Some(w) => w.subwords.to_vec(),
            None    => vec![],
        }
    }

    fn ans(&mut self) -> Result<Vec<Box<dyn Subword>>, ExecError> {
        let alts = self.get_alternative_subwords();
        if ! alts.is_empty() {
            Ok(alts)
        }else{
            Ok(vec![])
        }
    }

    fn check(&mut self) -> Result<(), ExecError> {
        if self.param.name.is_empty() || ! utils::is_param(&self.param.name) {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }
        if self.unknown.len() > 0 
        && ! self.unknown.starts_with(",") {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }
        Ok(())
    }

    fn index_replace(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
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
        //sw.replace = None;
        //sw.substr = None;
        sw.remove = None;
        sw.value_check = None;
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

        if self.array.len() <= 1 || self.value_check.is_some() {
            self.text = self.optional_operation(self.text.clone(), core)?;
        }else {
            for i in 0..self.array.len() {
                self.array[i] = self.optional_operation(self.array[i].clone(), core)?;
            }
            self.text = self.array.join(" ");
        }
        Ok(())
    }

    fn optional_operation(&mut self, text: String, core: &mut ShellCore) -> Result<String, ExecError> {
        if let Some(op) = self.optional_operation.as_mut() {
            return op.exec(&self.param, &text, core);
        }

        /*if let Some(s) = self.substr.as_mut() {
            s.get_text(&text, core)
        }else*/ if let Some(v) = self.value_check.as_mut() {
            v.set(&self.param.name, &self.param.subscript, &text, core)
        }else if let Some(r) = self.remove.as_mut() {
            r.set(&text, core)
        }else if let Some(c) = &self.case_conv {
            c.get_text(&text, core)
        }else{
            Ok(text.clone())
        }
    }
}
