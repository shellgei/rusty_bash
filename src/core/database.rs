//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

//The methods of DataBase are distributed in database/database_*.rs files.
pub mod data;
mod database_appenders;
mod database_checkers;
mod database_getters;
mod database_print;
mod database_setters;
mod database_unsetters;
mod database_initializers;

use self::data::array::ArrayData;
use self::data::array_int::IntArrayData;
use self::data::uninit::Uninit;
use self::data::assoc::AssocData;
use self::data::assoc_int::IntAssocData;
use self::data::single::SingleData;
use self::data::single_int::IntData;
use self::data::Data;
use crate::elements::command::function_def::FunctionDefinition;
use crate::error::exec::ExecError;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct DataBase {
    pub flags: String,
    params: Vec<HashMap<String, Box<dyn Data>>>,
    pub position_parameters: Vec<Vec<String>>,
    pub functions: HashMap<String, FunctionDefinition>,
    pub exit_status: i32,
    pub last_arg: String,
    pub hash_counter: HashMap<String, usize>,
}

impl DataBase {
    pub fn new() -> DataBase {
        let mut data = DataBase {
            params: vec![HashMap::new()],
            position_parameters: vec![vec![]],
            flags: "B".to_string(),
            ..Default::default()
        };

        data.initialize().unwrap();
        data
    }

    pub fn get_target_layer(&mut self, name: &str, layer: Option<usize>) -> usize {
        match layer {
            Some(n) => n,
            None => self.solve_layer(name),
        }
    }

    fn solve_layer(&mut self, name: &str) -> usize {
        self.get_layer_pos(name).unwrap_or(0)
    }

    pub fn push_local(&mut self) {
        self.params.push(HashMap::new());
    }

    pub fn pop_local(&mut self) {
        self.params.pop();
    }

    pub fn init(&mut self, name: &str, layer: usize) {
        if let Some(d) = self.params[layer].get_mut(name) {
            d.clear();
        }
    }

    pub fn int_to_str_type(&mut self, name: &str, layer: usize) -> Result<(), ExecError> {
        let layer_len = self.params.len();
        for ly in layer..layer_len {
            if let Some(v) = self.params[ly].get_mut(name) {
                let _ = v.unset_flag('i');
            }
        }

        if let Some(d) = self.params[layer].get_mut(name) {
            let new_d = d.get_str_type();
            self.params[layer].insert(name.to_string(), new_d);
        }

        Ok(())
    }
}
