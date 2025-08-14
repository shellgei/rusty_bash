//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use crate::elements::command::function_def::FunctionDefinition;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Default)]
pub struct DataBase {
    pub position_parameters: Vec<Vec<String>>,
    parameters: Vec<HashMap<String, String>>,
    pub functions: HashMap<String, FunctionDefinition>,
}

impl DataBase {
    pub fn new() -> Self {
        Self {
            position_parameters: vec![vec!["sush".to_string()]],
            parameters: vec![HashMap::new()],
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

        for params in self.parameters.iter().rev() {
            if params.contains_key(name) {
                return Ok(params[name].clone());
            }
        }

        if ! self.parameters[0].contains_key(name) {
            if let Ok(val) = env::var(name) {
                self.set_param(name, &val, None)?;
            }
        }

        let ans = match self.parameters[0].get(name) {
            Some(val) => val,
            None      => "",
        }.to_string();

        Ok(ans)
    }

    fn solve_set_layer(&mut self, name: &str, layer: Option<usize>) -> usize {
        if layer.is_some() {
            return layer.unwrap();
        }

        for (i, params) in self.parameters.iter().enumerate().rev() {
            if params.contains_key(name) {
                return i;
            }
        }

        return 0;
    }

    pub fn set_param(&mut self, name: &str, val: &str,
                     layer: Option<usize>) -> Result<(), ExecError> {
        let layer = self.solve_set_layer(name, layer);
        self.parameters[layer].insert(name.to_string(), val.to_string());
        if layer == 0 && env::var(name).is_ok() {
            env::set_var(name, val.to_string());
        }
        Ok(())
    }

    pub fn get_param_layer_ref(&self, layer: usize) -> &HashMap<String, String> {
        &self.parameters[layer]
    }

    pub fn push_local(&mut self) {
        self.parameters.push(HashMap::new());
    }   

    pub fn pop_local(&mut self) {
        self.parameters.pop();
    }

    pub fn get_layer_num(&mut self) -> usize {
        self.parameters.len()
    }
}
