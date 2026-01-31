//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

mod data;

use self::data::Data;
use self::data::random::RandomVar;
use self::data::srandom::SRandomVar;
use self::data::single::SingleData;
use crate::error::exec::ExecError;
use crate::elements::command::function_def::FunctionDefinition;
use std::collections::{HashMap, HashSet};
use std::env;

#[derive(Debug, Default)]
pub struct DataBase {
    pub position_parameters: Vec<Vec<String>>,
    params: Vec<HashMap<String, Box::<dyn Data>>>,
    pub functions: HashMap<String, FunctionDefinition>,
}

impl DataBase {
    pub fn new() -> Self {
        let mut ans = Self {
            position_parameters: vec![vec!["sush".to_string()]],
            params: vec![HashMap::new()],
            ..Default::default()
        };

        ans.params[0].insert("RANDOM".to_string(), Box::new(RandomVar::new()));
        ans.params[0].insert("SRANDOM".to_string(), Box::new(SRandomVar::new()));
        ans
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
                return params.get_mut(name).unwrap().get_as_single();
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
        if let Some(ly) = layer {
            return ly;
        }

        self.get_layer_pos(name).unwrap_or(0)
    }

    pub fn get_param_keys(&mut self) -> Vec<String> {
        let mut keys = HashSet::new();
        for layer in &self.params {
            layer.keys()
                 .for_each(|k| { keys.insert(k); });
        }
        let mut ans = keys.iter()
                          .map(|c| c.to_string())
                          .collect::<Vec<String>>();
        ans.sort();
        ans
    }

    pub fn get_func_keys(&mut self) -> Vec<String> {
        let mut keys = self.functions
                           .keys()
                           .map(|c| c.to_string())
                           .collect::<Vec<String>>();
        keys.sort();
        keys 
    }

    pub fn print_params_and_funcs(&mut self) {
        self.get_param_keys()
            .into_iter()
            .for_each(|k| self.print_param(&k));
        self.get_func_keys()
            .into_iter()
            .for_each(|k| { self.print_func(&k); });
    }

    pub fn print_param(&mut self, name: &str) {
        if let Some(layer) = self.get_layer_pos(name) {
            if let Some(d) = self.params[layer].get_mut(name) {
                let body = d.get_fmt_string();
                println!("{name}={body}");
            }
        }
    }

    pub fn print_func(&mut self, name: &str) -> bool {
        if let Some(f) = self.functions.get_mut(name) {
            println!("{}", &f.text);
            return true;
        }
        false
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
            unsafe{env::set_var(name, val)};
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

    pub fn get_flags(&mut self, name: &str) -> &str {
        match self.get_layer_pos(name) {
            Some(n) => self.params[n].get_mut(name).unwrap().get_flags(),
            None => "",
        }
    }

    pub fn has_flag(&mut self, name: &str, flag: char) -> bool {
        self.get_flags(name).contains(flag)
    }

    pub fn unset(&mut self, name: &str) -> Result<(), ExecError> {
        if ! self.unset_var(name)? {
            self.functions.remove(name);
        }
        Ok(())
    }

    pub fn unset_var(&mut self, name: &str)
                     -> Result<bool, ExecError> {
        let num = self.params.len();
        for layer in (0..num).rev() {
            if self.unset_var_layer(layer, name)? {
                return Ok(true)
            }
        }
        Ok(false)
    }

    fn unset_var_layer(&mut self, layer: usize, name: &str) -> Result<bool, ExecError> {
        if self.has_flag(name, 'r') {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }

        if ! self.params[layer].contains_key(name) {
            return Ok(false)
        }

        self.params[layer].remove(name);

        if layer == 0 {
            unsafe{env::remove_var(name)};
        }
        Ok(true)
    }
}
