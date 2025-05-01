//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use crate::elements::command::function_def::FunctionDefinition;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Default)]
pub struct Data {
    pub parameters: HashMap<String, String>,
    pub functions: HashMap<String, FunctionDefinition>,
}

impl Data {
    pub fn get_param(&mut self, name: &str) -> Result<String, ExecError> {
        if ! self.parameters.contains_key(name) {
            if let Ok(val) = env::var(name) {
                self.set_param(name, &val)?;
            }
        }

        let ans = match self.parameters.get(name) {
            Some(val) => val,
            None      => "",
        }.to_string();

        Ok(ans)
    }

    pub fn set_param(&mut self, name: &str, val: &str) -> Result<(), ExecError> {
        self.parameters.insert(name.to_string(), val.to_string());
        Ok(())
    }
}
