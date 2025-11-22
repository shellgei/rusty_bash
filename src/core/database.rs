//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

mod data;

use self::data::Data;
use self::data::single::SingleData;
use crate::error::exec::ExecError;
use crate::elements::command::function_def::FunctionDefinition;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Default)]
pub struct DataBase {
    pub position_parameters: Vec<Vec<String>>,
    params: Vec<HashMap<String, Box::<dyn Data>>>,
    pub functions: HashMap<String, FunctionDefinition>,
}

impl DataBase {
    pub fn new() -> Self {
        Self {
            position_parameters: vec![vec!["sush".to_string()]],
            params: vec![HashMap::new()],
            ..Default::default()
        }
    }

    pub fn get_param(&mut self, name: &str) -> Result<String, ExecError> {
        if let Ok(n) = name.parse::<usize>() {
            let layer = &self.position_parameters.last().unwrap();
            if  n < layer.len() {
                return Ok(layer[n].to_string());
            }
            return Ok("".to_string());
        }

        for params in self.params.iter_mut().rev() {
            if params.contains_key(name) {
                return Ok(params.get_mut(name).unwrap().get_as_single()?);
            }
        }

        if ! self.params[0].contains_key(name) {
            if let Ok(val) = env::var(name) {
                self.set_param(name, &val, None)?;
            }
        }

        let ans = match self.params[0].get_mut(name) {
            Some(val) => val.get_as_single()?,
            None      => String::new(),
        };

        Ok(ans)
    }

    pub fn solve_set_layer(&mut self, name: &str,
                           layer: Option<usize>) -> usize {
        if layer.is_some() {
            return layer.unwrap();
        }

        self.get_layer_pos(name).unwrap_or(0)
    }

    pub fn get_layer_pos(&mut self, name: &str) -> Option<usize> {
        let num = self.params.len();
        (0..num)
            .rev()
            .find(|&layer| self.params[layer].contains_key(name))
    }

    pub fn set_param(&mut self, name: &str, val: &str,
                     layer: Option<usize>) -> Result<(), ExecError> {
        let layer = self.solve_set_layer(name, layer);

        if let Some(d) = self.params[layer].get_mut(name) {
            d.set_as_single(name, val)?;
        }else{
            self.params[layer].insert(name.to_string(),
                                      Box::new(SingleData::from(val)));
        }

        if layer == 0 && env::var(name).is_ok() {
            env::set_var(name, val.to_string());
        }
        Ok(())
    }

    pub fn get_param_layer_ref(&mut self, layer: usize) -> &mut HashMap<String, Box::<dyn Data>> {
        &mut self.params[layer]
    }

    pub fn push_local(&mut self) {
        self.params.push(HashMap::new());
    }

    pub fn pop_local(&mut self) {
        self.params.pop();
    }

    pub fn get_layer_num(&mut self) -> usize {
        self.params.len()
    }

    pub fn set_flag(&mut self, name: &str, flag: char, layer: usize) {
        if let Some(d) = self.params[layer].get_mut(name) {
            d.set_flag(flag);
        }
    }
}
