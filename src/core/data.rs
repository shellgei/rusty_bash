//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::elements::command::function_def::FunctionDefinition;
use std::env;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Data {
    pub flags: String,
    pub parameters: Vec<HashMap<String, String>>,
    pub arrays: Vec<HashMap<String, Vec<String>>>,
    pub position_parameters: Vec<Vec<String>>,
    pub aliases: HashMap<String, String>,
    pub functions: HashMap<String, FunctionDefinition>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            flags: String::new(),
            parameters: vec![HashMap::new()],
            arrays: vec![HashMap::new()],
            position_parameters: vec![vec![]],
            aliases: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_param(&mut self, key: &str) -> String {
        if key == "-" {
            return self.flags.clone();
        }

        if let Some(n) = self.get_position_param_pos(key) {
            let layer = self.position_parameters.len();
            return self.position_parameters[layer-1][n].to_string();
        }

        let num = self.parameters.len();
        if num > 0 {
            for layer in (0..num).rev() {
                match self.parameters[layer].get(key) {
                    Some(val) => return val.to_string(),
                    None      => {},
                }
            }
        }

        if self.parameters[0].get(key) == None {
            if let Ok(val) = env::var(key) {
                self.set_param(key, &val);
            }
        }

        match self.parameters[0].get(key) {
            Some(val) => val.to_string(),
            None      => "".to_string(),
        }
    }

    pub fn get_array(&mut self, key: &str, pos: &str) -> String {
        let num = self.parameters.len();
        for layer in (0..num).rev()  {
            if  self.arrays[layer].get(key) == None {
                continue;
            }
    
            match self.arrays[layer].get(key) {
                Some(a) => {
                    if pos == "@" {
                        return a.join(" ");
                    }
    
                    match pos.parse::<usize>() {
                        Ok(n) => {
                            if n < a.len() {
                                return a[n].clone();
                            }
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
        }

        "".to_string()
    }

    fn get_position_param_pos(&self, key: &str) -> Option<usize> {
        if ! (key.len() == 1 && "0" <= key && key <= "9") {
            return None;
        }

        let n = key.parse::<usize>().unwrap();
        let layer = self.position_parameters.len();
        if n < self.position_parameters[layer-1].len() {
            Some(n)
        }else{
            None
        }
    }

    fn set_layer_param(&mut self, key: &str, val: &str, layer: usize) {
        match env::var(key) {
            Ok(_) => env::set_var(key, val),
            _     => {},
        }

        self.parameters[layer].insert(key.to_string(), val.to_string());
    }

    pub fn set_param(&mut self, key: &str, val: &str) {
        self.set_layer_param(key, val, 0);
    }

    pub fn set_local_param(&mut self, key: &str, val: &str) {
        let layer = self.parameters.len();
        self.set_layer_param(key, val, layer-1);
    }

    pub fn set_layer_array(&mut self, key: &str, vals: &Vec<String>, layer: usize) {
        self.arrays[layer].insert(key.to_string(), vals.to_vec());
    }

    pub fn set_array(&mut self, key: &str, vals: &Vec<String>) {
        self.set_layer_array(key, vals, 0);
    }

    pub fn set_local_array(&mut self, key: &str, vals: &Vec<String>) {
        let layer = self.arrays.len();
        self.set_layer_array(key, vals, layer-1);
    }

    pub fn push_local(&mut self) {
        self.parameters.push(HashMap::new());
        self.arrays.push(HashMap::new());
    }

    pub fn pop_local(&mut self) {
        self.parameters.pop();
        self.arrays.pop();
    }
}
