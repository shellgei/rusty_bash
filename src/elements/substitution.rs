//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod array;
pub mod subscript;
pub mod variable;
pub mod value;

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use crate::elements::word::Word;
use self::value::Value;
use self::variable::Variable;
use crate::elements::word::WordMode;

#[derive(Debug, Clone, Default)]
pub struct Substitution {
    pub text: String,
    pub left_hand: Variable,
    pub right_hand: Value,
    append: bool,
    lineno: usize,
    pub has_right: bool,
//    pub double_quoted: bool,
}

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore, layer: Option<usize>, declare: bool)
    -> Result<(), ExecError> {
        core.db.set_param("LINENO", &self.lineno.to_string(), None)?;
        self.right_hand.eval(core, &self.left_hand.name, self.append)?;

        if declare && self.right_hand.evaluated_array.is_some() {
            self.left_hand.index = None;
        }

        self.set_to_shell(core, layer)
    }

    pub fn reparse(&mut self, core: &mut ShellCore) //TODO: solve this confusion
    -> Result<(), ExecError> {
        let mut f = Feeder::new(&self.text);

        let text = if let Ok(Some(s)) = Word::parse(&mut f, core, Some(WordMode::ReparseOfSubstitutionArray)) {
            if ! f.is_empty() {
                return Err(ExecError::InvalidName(self.text.clone()));
            }
    
            s.eval_as_value(core)?
        }else{
            return Err(ExecError::InvalidName(self.text.clone()));
        };
    
        let mut f = Feeder::new(&text.replace("~", "\\~"));
        if let Ok(Some(s)) = Substitution::parse(&mut f, core, true) {
            if ! f.is_empty() {
                return Err(ExecError::InvalidName(text));
            }
    
            *self = s;
            return Ok(());
        }
    
        Err(ExecError::InvalidName(text))
    }

    fn set_whole_array(&mut self, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
        if self.right_hand.evaluated_array.is_none() {
            return Err(ExecError::Other("no array and no index".to_string()));
        }

        let a = self.right_hand.evaluated_array.as_ref().unwrap();

        if a.is_empty() && ! self.append {
            if core.db.is_assoc(&self.left_hand.name) {
                core.db.set_assoc(&self.left_hand.name, Some(layer), true)?;
            }else{
                core.db.set_array(&self.left_hand.name, Some(vec![]), Some(layer))?;
            }
            return Ok(());
        }else if ! self.append {
            core.db.init(&self.left_hand.name, layer);
        }

        for e in a {
            core.db.set_param2(&self.left_hand.name, &e.0, &e.1, Some(layer))?;
        }
        Ok(())
    }

    fn set_array_elem(&mut self, core: &mut ShellCore, layer: usize, index: &String)
    -> Result<(), ExecError> {
        if index.is_empty() {
            return Err(ExecError::ArrayIndexInvalid(self.left_hand.text.clone()));
        }
        if let Some(v) = &self.right_hand.evaluated_string {
            if self.append {
                return core.db.append_param2(&self.left_hand.name, index, &v, Some(layer));
            }else {
                return core.db.set_param2(&self.left_hand.name, index, &v, Some(layer));
            }
        }

        let msg = format!("{}: cannot assign list to array member", &self.left_hand.text);
        Err(ExecError::Other(msg))
    }

    fn set_array(&mut self, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
        let rhs_is_array = self.right_hand.evaluated_array.is_some();

        match self.left_hand.get_index(core, rhs_is_array, self.append)? {
            Some(index) => self.set_array_elem(core, layer, &index),
            None        => self.set_whole_array(core, layer),
        }
    }

    fn set_single(&mut self, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
        let data = self.right_hand.evaluated_string.clone().unwrap();
        if self.append {
            core.db.append_param(&self.left_hand.name, &data, Some(layer))
        }else{
            core.db.set_param(&self.left_hand.name, &data, Some(layer))
        }
    }

    fn set_to_shell(&mut self, core: &mut ShellCore, layer: Option<usize>)
    -> Result<(), ExecError> {
        let layer = core.db.get_target_layer(&self.left_hand.name, layer);

        if self.right_hand.evaluated_string.is_some()
        && self.left_hand.index.is_none() {
            self.set_single(core, layer)
        }else{
            self.set_array(core, layer)
        }
    }

    fn eat_equal(&mut self, feeder: &mut Feeder) -> bool {
        if feeder.starts_with("+=") {
            self.append = true;
            self.text += &feeder.consume(2);
        }else if feeder.starts_with("=") {
            self.text += &feeder.consume(1);
        }else{
            return false;
        }

        true
    }

    fn eat_left_hand(&mut self, feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<bool, ParseError> {
        self.left_hand = match Variable::parse(feeder, core)? {
            Some(a) => a,
            None => return Ok(false),
        };
        self.text = self.left_hand.text.clone();
        self.lineno = self.left_hand.lineno;
        Ok(true)
    }

    /* for the case 'declare "A=B"' */
    pub fn parse_double_quoted(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        let text = if let Some(s) = Word::parse(feeder, core, None)? {
            let mut txt = s.text;
            txt.remove(0);
            txt.pop();
            txt
        }else{
            return Ok(None);
        };
    
        let mut f = Feeder::new(&text.replace("~", "\\~"));
        return Substitution::parse(&mut f, core, true);
    }

    pub fn parse_as_arg(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        if feeder.starts_with("\"") 
        && ! feeder.starts_with("\"-") {
            return Self::parse_double_quoted(feeder, core);
        }

        let mut ans = Self::default();
        if ! ans.eat_left_hand(feeder, core)? {
            return Ok(None);
        }

        if ! ans.eat_equal(feeder) {
            ans.has_right = false;
            return Ok(Some(ans));
        }

        ans.has_right = true;
        if let Some(a) = Value::parse(feeder, core, false)? {
            ans.text += &a.text.clone();
            ans.right_hand = a;
        }

        Ok(Some(ans))
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore, permit_space: bool)
    -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();

        feeder.set_backup();
        match ans.eat_left_hand(feeder, core) {
            Ok(true) => {},
            Ok(false) => {
                feeder.rewind();
                return Ok(None)
            },
            Err(e) => {
                feeder.rewind();
                return Err(e);
            },
        }

        if ! ans.eat_equal(feeder) {
            feeder.rewind();
            return Ok(None);
        }
        feeder.pop_backup();

        ans.has_right = true;
        if let Some(a) = Value::parse(feeder, core, permit_space)? {
            ans.text += &a.text.clone();
            ans.right_hand = a;
        }

        Ok(Some(ans))
    }
}
