//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

//The methods of DataBase are distributed in database/database_*.rs files.
pub mod data;
mod database_checker;
mod database_getter;
mod database_setter;
mod database_initializer;

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
use crate::env;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct DataBase {
    pub flags: String,
    pub params: Vec<HashMap<String, Box<dyn Data>>>,
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

        database_initializer::initialize(&mut data).unwrap();
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

    pub fn unset_var(&mut self, name: &str, called_layer: Option<usize>) {
        if let Some(layer) = called_layer {
            if layer == 0 {
                return;
            }
            self.params[layer].remove(name);

            env::set_var(name, "");
            for layer in self.params.iter_mut() {
                if let Some(val) = layer.get_mut(name) {
                    *val = Box::new( Uninit::new("") );
                }
            }

            return;
        }

        env::remove_var(name);

        for layer in &mut self.params {
            layer.remove(name);
        }
    }

    pub fn unset_function(&mut self, name: &str) {
        self.functions.remove(name);
    }

    pub fn unset(&mut self, name: &str, called_layer: Option<usize>) {
        self.unset_var(name, called_layer);
        self.unset_function(name);
    }

    pub fn unset_array_elem(&mut self, name: &str, key: &str) -> Result<(), ExecError> {
        if self.is_single(name) && (key == "0" || key == "@" || key == "*") {
            self.unset_var(name, None);
            return Ok(());
        }

        for layer in &mut self.params {
            if let Some(d) = layer.get_mut(name) {
                d.remove_elem(key)?;
            }
        }
        Ok(())
    }

    pub fn print(&mut self, name: &str) {
        if let Some(d) = self.get_ref(name) {
            d.print_with_name(name, false);
        } else if let Some(f) = self.functions.get(name) {
            println!("{}", &f.text);
        }
    }

    pub fn declare_print(&mut self, name: &str) {
        if let Some(d) = self.get_ref(name) {
            d.print_with_name(name, true);
        } else if let Some(f) = self.functions.get(name) {
            println!("{}", &f.text);
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
