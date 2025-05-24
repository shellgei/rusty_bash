//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

mod data;
mod getter;
mod setter;

use crate::{env, exit};
use crate::elements::command::function_def::FunctionDefinition;
use std::collections::{HashMap, HashSet};
use crate::utils;
use crate::utils::file_check;
use crate::error::exec::ExecError;
use self::data::Data;
use self::data::assoc::AssocData;
use self::data::single::SingleData;
use self::data::array::ArrayData;
//use self::data::special::SpecialData;

#[derive(Debug, Default)]
pub struct DataBase {
    pub flags: String,
    pub params: Vec<HashMap<String, Box<dyn Data>>>,
    pub param_options: Vec<HashMap<String, String>>,
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
            param_options: vec![HashMap::new()],
            position_parameters: vec![vec![]],
            flags: "B".to_string(),
            ..Default::default()
        };

        setter::initialize(&mut data).unwrap();
        data
    }

    fn name_check(name: &str) -> Result<(), ExecError> {
        if ! utils::is_param(name) {
            return Err(ExecError::VariableInvalid(name.to_string()));
        }
        Ok(())
    }

    pub fn is_readonly(&mut self, name: &str) -> bool {
        self.has_flag(name, 'r')
    }

    fn rsh_cmd_check(&mut self, cmds: &Vec<String>) -> Result<(), ExecError> {
        for c in cmds {
            if c.contains('/') {
                let msg = format!("{}: restricted", &c);
                return Err(ExecError::Other(msg));
            }

            if file_check::is_executable(&c) {
                let msg = format!("{}: not found", &c);
                return Err(ExecError::Other(msg));
            }
        }
        Ok(())
    }

    fn rsh_check(&mut self, name: &str) -> Result<(), ExecError> {
        if ["SHELL", "PATH", "ENV", "BASH_ENV"].contains(&name) {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }
        Ok(())
    }

    fn write_check(&mut self, name: &str) -> Result<(), ExecError> {
        if self.has_flag(name, 'r') {
            self.exit_status = 1;
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }

        Ok(())
    }

    pub fn get_param(&mut self, name: &str) -> Result<String, ExecError> {
        Self::name_check(name)?;

        if let Some(val) = getter::special_param(self, name) {
            return Ok(val);
        }

        if name == "@" || name == "*" {   // $@ should return an array in a double quoted
            return getter::connected_position_params(self, name == "*");  // subword. Therefore another 
        }                                                   //access method should be used there. 

        if let Ok(n) = name.parse::<usize>() {
            return getter::position_param(self, n);
        }

        if let Some(v) = self.get_ref(name) {
            if v.is_special() {
                return Ok(v.get_as_single()?);
            }
            if v.is_single_num() {
                let val = v.get_as_single_num()?;//.unwrap_or_default();
                return Ok(val.to_string());
            }else{
                let val = v.get_as_single().unwrap_or_default();
                return Ok(val);
            }
        }

        if let Ok(v) = env::var(name) {
            let _ = self.set_param(name, &v, Some(0));
            return Ok(v);
        }

        Ok("".to_string())
    }

    pub fn get_param2(&mut self, name: &str, index: &String) -> Result<String, ExecError> {
        match index.is_empty() {
            true  => self.get_param(&name),
            false => self.get_array_elem(&name, &index),
        }
}

    pub fn get_array_elem(&mut self, name: &str, pos: &str) -> Result<String, ExecError> {
        Self::name_check(name)?;
        getter::array_elem(self, name, pos)
    }

    pub fn has_value_layer(&mut self, name: &str, layer: usize) -> bool {
        if let Some(_) = self.params[layer].get(name) {
            return true;
        }
        false
    }

    pub fn has_value(&mut self, name: &str) -> bool {
        if let Ok(n) = name.parse::<usize>() {
            let layer = self.position_parameters.len() - 1;
            return n < self.position_parameters[layer].len();
        }

        let num = self.params.len();
        for layer in (0..num).rev()  {
            if self.has_value_layer(name, layer) {
                return true;
            }
        }
        false
    }

    pub fn has_array_value(&mut self, name: &str, index: &str) -> bool {
        let num = self.params.len();
        for layer in (0..num).rev()  {
            if let Some(e) = self.params[layer].get(name) {
                let mut a = e.clone();
                return a.get_as_array_or_assoc(index).is_ok();
            }
        }
        false
    }

    pub fn len(&mut self, name: &str) -> usize {
        if let Some(d) = self.get_ref(name) {
            return d.len();
        }
        0
    }

    pub fn get_array_all(&mut self, name: &str) -> Vec<String> {
        let layer = self.position_parameters.len() - 1;
        if name == "@" {
            return self.position_parameters[layer].clone();
        }

        match self.get_ref(name) {
            Some(d) => {
                if let Ok(v) = d.get_all_as_array() {
                    return v;
                }
                vec![]
            },
            None => vec![],
        }
    }

    pub fn get_indexes_all(&mut self, name: &str) -> Vec<String> {
        let layer = self.position_parameters.len() - 1;
        if name == "@" {
            return self.position_parameters[layer].clone();
        }

        match self.get_ref(name) {
            Some(d) => {
                match d.get_all_indexes_as_array() {
                    Ok(v) => v,
                    _ => vec![],
                }
            },
            None => vec![],
        }
    }

    pub fn is_array(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => return d.is_array(),
            _ => false,
        }
    }

    pub fn is_assoc(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => d.is_assoc(),
            None => false,
        }
    }

    pub fn is_single(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => return d.is_single(),
            _ => false,
        }
    }

    pub fn is_single_num(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => return d.is_single_num(),
            _ => false,
        }
    }

    pub fn get_position_params(&self) -> Vec<String> {
        match self.position_parameters.last() {
            Some(v) => v[1..].to_vec(),
            _       => vec![],
        }
    }

    pub fn has_flag(&mut self, name: &str, flag: char) -> bool {
        let layer = self.param_options.len() - 1;
        match self.param_options[layer].get(name) {
            None => false,
            Some(e) => e.contains(flag),
        }
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

    pub fn get_layer_pos(&mut self, name: &str) -> Option<usize> {
        let num = self.params.len();
        for layer in (0..num).rev()  {
            if self.params[layer].get(name).is_some() {
                return Some(layer);
            }
        }
        None
    }

    pub fn init_as_num(&mut self, name: &str, value: &str, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        if self.flags.contains('r') {
            if name == "BASH_CMDS" {
                self.rsh_cmd_check(&vec![value.to_string()])?;
            }
            self.rsh_check(name)?;
        }

        let layer = self.get_target_layer(name, layer);
        match self.param_options[layer].get_mut(name) {
            Some(e) => *e += "i",
            None => {self.param_options[layer].insert(name.to_string(), "i".to_string());},
        }
        SingleData::init_as_num(&mut self.params[layer], name, value)
    }

    pub fn set_param(&mut self, name: &str, val: &str, layer: Option<usize>)
    -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        if self.flags.contains('r') {
            if name == "BASH_CMDS" {
                self.rsh_cmd_check(&vec![val.to_string()])?;
            }
            self.rsh_check(name)?;
        }

        if name == "BASH_ARGV0" {
            let n = layer.unwrap_or(self.get_layer_num() - 1);
            self.position_parameters[n][0] = val.to_string();
        }

        if ! self.flags.contains('r')
        && self.flags.contains('a') {
            env::set_var(name, "");
        }

        let layer = self.get_target_layer(name, layer);
        SingleData::set_value(&mut self.params[layer], name, val)
    }

    pub fn append_param(&mut self, name: &str, val: &str, layer: Option<usize>)
    -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        if self.flags.contains('r') {
            if name == "BASH_CMDS" {
                self.rsh_cmd_check(&vec![val.to_string()])?;
            }
            self.rsh_check(name)?;
        }

        if name == "BASH_ARGV0" {
            let n = layer.unwrap_or(self.get_layer_num() - 1);
            self.position_parameters[n][0] += val;
        }


        if ! self.flags.contains('r')
        && self.flags.contains('a')
        && ! env::var(name).is_ok() {
            env::set_var(name, "");
        }

        let layer = self.get_target_layer(name, layer);
        SingleData::append_value(&mut self.params[layer], name, val)
    }

    pub fn set_param2(&mut self, name: &str, index: &String, val: &String,
                      layer: Option<usize>) -> Result<(), ExecError> {
        if index.is_empty() {
            return self.set_param(name, val, layer);
        }

        if self.is_array(name) {
            if let Ok(n) = index.parse::<usize>() {
                self.set_array_elem(&name, val, n, layer)?;
            }
        }else if self.is_assoc(name) {
            self.set_assoc_elem(&name, &index, val, layer)?;
        }else{
            match index.parse::<usize>() {
                Ok(n) => {self.set_array_elem(&name, val, n, layer)?;},
                _ => {self.set_assoc_elem(&name, &index, val, layer)?;},
            }
        }
        Ok(())
    }

    pub fn append_param2(&mut self, name: &str, index: &String, val: &String,
                      layer: Option<usize>) -> Result<(), ExecError> {
        if index.is_empty() {
            return self.append_param(name, val, layer);
        }

        if self.is_array(name) {
            if let Ok(n) = index.parse::<usize>() {
                self.append_array_elem(&name, val, n, layer)?;
            }
        }else if self.is_assoc(name) {
            self.append_assoc_elem(&name, &index, val, layer)?;
        }else{
            match index.parse::<usize>() {
                Ok(n) => {self.append_array_elem(&name, val, n, layer)?;},
                _ => {self.append_assoc_elem(&name, &index, val, layer)?;},
            }
        }
        Ok(())
    }

    pub fn set_array_elem(&mut self, name: &str, val: &String, pos: usize, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        if self.flags.contains('r') {
            if name == "BASH_CMDS" {
                self.rsh_cmd_check(&vec![val.to_string()])?;
            }
            self.rsh_check(name)?;
        }

        let layer = self.get_target_layer(name, layer);
        ArrayData::set_elem(&mut self.params[layer], name, pos, val)
    }

    pub fn append_array_elem(&mut self, name: &str, val: &String, pos: usize, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        if self.flags.contains('r') {
            if name == "BASH_CMDS" {
                self.rsh_cmd_check(&vec![val.to_string()])?;
            }
            self.rsh_check(name)?;
        }

        let layer = self.get_target_layer(name, layer);
        ArrayData::append_elem(&mut self.params[layer], name, pos, val)
    }

    pub fn set_assoc_elem(&mut self, name: &str, key: &String, val: &String, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        if self.flags.contains('r') {
            if name == "BASH_CMDS" {
                self.rsh_cmd_check(&vec![val.to_string()])?;
            }
            self.rsh_check(name)?;
        }

        let layer = self.get_target_layer(name, layer);
        AssocData::set_elem(&mut self.params[layer], name, key, val)
    }

    pub fn append_assoc_elem(&mut self, name: &str, key: &String, val: &String, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        if self.flags.contains('r') {
            if name == "BASH_CMDS" {
                self.rsh_cmd_check(&vec![val.to_string()])?;
            }
            self.rsh_check(name)?;
        }

        let layer = self.get_target_layer(name, layer);
        AssocData::append_elem(&mut self.params[layer], name, key, val)
    }

    pub fn set_array(&mut self, name: &str, v: Vec<String>, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        if self.flags.contains('r') {
            if name == "BASH_CMDS" {
                for val in &v{
                    self.rsh_cmd_check(&vec![val.to_string()])?;
                }
            }
            self.rsh_check(name)?;
        }

        let layer = self.get_target_layer(name, layer);
        ArrayData::set_new_entry(&mut self.params[layer], name, v)
    }

    pub fn set_assoc(&mut self, name: &str, layer: Option<usize>) -> Result<(), ExecError> {
        Self::name_check(name)?;
        self.write_check(name)?;
        if self.flags.contains('r') {
            self.rsh_check(name)?;
        }

        let layer = self.get_target_layer(name, layer);
        AssocData::set_new_entry(&mut self.params[layer], name)
    }

    pub fn push_local(&mut self) {
        self.params.push(HashMap::new());
        match self.param_options.last() {
            Some(e) => self.param_options.push(e.clone()),
            None => exit::internal("error: DataBase::push_local"),
        }
    }

    pub fn pop_local(&mut self) {
        self.params.pop();
        self.param_options.pop();
    }

    pub fn get_layer_num(&mut self) -> usize { self.params.len() }

    pub fn get_keys(&mut self) -> Vec<String> {
        let mut keys = HashSet::new();
        for layer in &self.params {
            layer.keys().for_each(|k| {keys.insert(k);} );
        }
        for f in &self.functions {
            keys.insert(f.0);
        }
        let mut ans: Vec<String> = keys.iter().map(|c| c.to_string()).collect();
        ans.sort();
        ans
    }

    pub fn init(&mut self, name: &str, layer: usize) {
        if let Some(d) = self.params[layer].get_mut(name) {
            d.clear();
        }
    }

    pub fn unset_var(&mut self, name: &str) {
        env::remove_var(name);

        for layer in &mut self.params {
            layer.remove(name);
        }
        for layer in &mut self.param_options {
            layer.remove(name);
        }
    }

    pub fn unset_function(&mut self, name: &str) {
        self.functions.remove(name);
    }

    pub fn unset(&mut self, name: &str) {
        self.unset_var(name);
        self.unset_function(name);
    }

    pub fn unset_array_elem(&mut self, name: &str, key: &str) {
        for layer in &mut self.params {
            if let Some(d) = layer.get_mut(name) {
                let _ = d.remove_elem(key);
            }
        }
    }

    pub fn set_flag(&mut self, name: &str, flag: char) {
        setter::flag(self, name, flag)
    }

    pub fn print(&mut self, name: &str) {
        if let Some(d) = self.get_ref(name) {
            d.print_with_name(name);
        }else if let Some(f) = self.functions.get(name) {
            println!("{}", &f.text);
        }
    }

    fn get_ref(&mut self, name: &str) -> Option<&mut Box<dyn Data>> {
        let num = self.params.len();
        for layer in (0..num).rev() {
            if self.params[layer].get_mut(name).is_some() {
                return self.params[layer].get_mut(name);
            }
        }
        None
    }

    pub fn get_ifs_head(&mut self) -> String {
        let ifs = self.get_param("IFS").unwrap_or(" ".to_string());
        match ifs.as_str() {
            "" => "".to_string(), 
            s => s.chars().next().unwrap().to_string(),
        }
    }
}
