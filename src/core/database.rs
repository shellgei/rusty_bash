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
    pub fn get_param(&mut self, name: &str) -> Result<String, ExecError> {
        if let Ok(n) = name.parse::<usize>() {
            let layer = &self.position_parameters.last().unwrap();
            if  n < layer.len() {
                return Ok(layer[n].to_string());
            }
            return Ok("".to_string());
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

    pub fn set_param(&mut self, name: &str, val: &str,
                     layer: Option<usize>) -> Result<(), ExecError> {
        self.parameters[0].insert(name.to_string(), val.to_string());
        Ok(())
    }
}
