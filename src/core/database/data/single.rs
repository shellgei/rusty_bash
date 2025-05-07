//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::utils;
use crate::core::HashMap;
use crate::error::exec::ExecError;
use std::env;
use super::Data;

#[derive(Debug, Clone)]
enum Body {
    Str(String),
    Num(Option<isize>),
}

impl Body {
    fn to_string(&self) -> String {
        match &self {
            Self::Str(s) => s.clone(),
            Self::Num(None) => "".to_string(),
            Self::Num(Some(n)) => n.to_string(),
        }
    }

    fn clear(&mut self) {
        *self = match &self {
            Self::Str(_) => Self::Str(String::new()),
            _ => Self::Num(None),
        }
    }

    fn get_as_num(&mut self) -> Result<isize, ExecError> {
        match self {
            Self::Str(s) => Err(ExecError::InvalidNumber(s.to_string())),
            Self::Num(None) => Ok(0),
            Self::Num(Some(n)) => Ok(*n),
        }
    }

    fn len(&self) -> usize {
        match &self {
            Self::Str(s) => s.chars().count(),
            Self::Num(None) => 0,
            Self::Num(Some(n)) => n.to_string().len(),
        }
    }

    fn set(&mut self, value: &str) -> Result<(), ExecError> {
        match self {
            Self::Str(_) => *self = Self::Str(value.to_string()),
            Self::Num(_) => {
                match value.parse::<isize>() {
                    Ok(n) => *self = Self::Num(Some(n)),
                    _ => return Err(ExecError::InvalidNumber(value.to_string())),
                }
            },
        }
        Ok(())
    }

    fn is_num(&self) -> bool {
        match self {
            Self::Num(_) => true,
            Self::Str(_) => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SingleData {
    body: Body,
}

impl From<&str> for SingleData {
    fn from(s: &str) -> Self {
        Self {
            body: Body::Str(s.to_string()),
        }
    }
}

impl Data for SingleData {
    fn boxed_clone(&self) -> Box<dyn Data> { Box::new(self.clone()) }
    fn print_body(&self) -> String { 
        utils::to_ansi_c(&self.body.to_string())
    }

    fn clear(&mut self) { self.body.clear(); }

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        self.body.set(value)
    }

    fn init_as_num(&mut self) -> Result<(), ExecError> {
        self.body = Body::Num(None);
        Ok(())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> { Ok(self.body.to_string()) }

    fn get_as_single_num(&mut self) -> Result<isize, ExecError> {
        self.body.get_as_num()
    }

    fn len(&mut self) -> usize { self.body.len() }
    fn is_single(&self) -> bool {true}
    fn is_single_num(&self) -> bool { self.body.is_num() }
}

impl SingleData {
    pub fn set_new_entry(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str, value: &str)-> Result<(), ExecError> {
        db_layer.insert( name.to_string(), Box::new(SingleData::from(value)) );
        Ok(())
    }

    pub fn set_value(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str, val: &str) -> Result<(), ExecError> {
        if env::var(name).is_ok() {
            env::set_var(name, val);
        }

        if db_layer.get(name).is_none() {
            SingleData::set_new_entry(db_layer, name, "")?;
        }

        let d = db_layer.get_mut(name).unwrap();

        if d.is_array() {
            return d.set_as_array("0", val);
        }
     
        d.set_as_single(val)
    }

    pub fn init_as_num(db_layer: &mut HashMap<String, Box<dyn Data>>,
        name: &str, value: &str)-> Result<(), ExecError> {
        let mut data = SingleData{body: Body::Num(None)};

        if value != "" {
            match value.parse::<isize>() {
                Ok(n) => data.body = Body::Num(Some(n)),
                Err(e) => {
                    return Err(ExecError::Other(e.to_string()));
                },
            }
        }

        db_layer.insert( name.to_string(), Box::new(data) );
        Ok(())
    }
}
