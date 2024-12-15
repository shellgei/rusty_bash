//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod variable;

use crate::core::data::variable::single::SingleData;
use crate::core::data::variable::special::SpecialData;
use crate::elements::command::function_def::FunctionDefinition;
use self::variable::{Value, Variable};
use std::{env, process};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Data {
    pub flags: String,
    parameters: Vec<HashMap<String, Variable>>,
    pub position_parameters: Vec<Vec<String>>,
    pub aliases: HashMap<String, String>,
    pub functions: HashMap<String, FunctionDefinition>,
    pub alias_memo: Vec<(String, String)>,
}

impl Data {
    pub fn new() -> Data {
        let mut data = Data {
            parameters: vec![HashMap::new()],
            position_parameters: vec![vec![]],
            flags: "B".to_string(),
            ..Default::default()
        };

        data.set_param("$", &process::id().to_string());
        data.set_param("BASHPID", &process::id().to_string());
        data.set_param("BASH_SUBSHELL", "0");
        data.set_param("?", "0");
        data.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()));

        data
    }

    pub fn get_param(&mut self, name: &str) -> String {
        if name == "-" {
            return self.flags.clone();
        }

        if name == "@" || name == "*" { // $@ should return an array in a double quoted
                                      // subword. Therefore another access method is used there. 
            return match self.position_parameters.last() {
                Some(a) => a[1..].join(" "),
                _       => "".to_string(),
            };
        }

        if let Some(n) = self.get_position_param_pos(name) {
            let layer = self.position_parameters.len();
            return self.position_parameters[layer-1][n].to_string();
        }

        match self.get_value(name) {
            Some(Value::Special(v)) => return v.data.to_string(),
            Some(Value::Single(v)) => return v.data.to_string(),
            Some(Value::Array(a)) => {
                match a.len() {
                    0 => return "".to_string(),
                    _ => return a.get(0).unwrap_or("".to_string()),
                }
            },
            _  => {},
        }

        match env::var(name) {
            Ok(v) => {
                self.set_layer_param(name, &v, 0);
                v
            },
            _ => "".to_string()
        }
    }

    pub fn get_array(&mut self, name: &str, pos: &str) -> String {
        match self.get_value(name) {
            Some(Value::Array(a)) => {
                if pos == "@" || pos == "*" {
                    return a.join(" ");
                } else if let Ok(n) = pos.parse::<usize>() {
                    return a.get(n).unwrap_or("".to_string());
                }
            },
            Some(Value::AssocArray(a)) => {
                if pos == "@" || pos == "*" {
                    let values = a.values();
                    return values.join(" ");
                }
                return a.get(pos).unwrap_or("".to_string());
            },
            Some(Value::Single(v)) => {
                match pos.parse::<usize>() {
                    Ok(0) => return v.data.to_string(),
                    Ok(_) => return "".to_string(),
                    _ => return v.data.to_string(), 
                }
            },
            _ => {},
        }
        "".to_string()
    }

    pub fn get_value(&mut self, key: &str) -> Option<Value> {
        let num = self.parameters.len();
        for layer in (0..num).rev()  {
            if let Some(v) = self.parameters[layer].get_mut(key) {
                return Some(v.get_value());
            }
        }
        None
    }

    pub fn has_value(&mut self, key: &str) -> bool {
        let num = self.parameters.len();
        for layer in (0..num).rev()  {
            if let Some(_) = self.parameters[layer].get(key) {
                return true;
            }
        }
        false
    }

    pub fn len(&mut self, key: &str) -> usize {
        match self.get_value(key) {
            Some(Value::Array(a)) => a.len(),
            _ => 0,
        }
    }

    pub fn get_array_all(&mut self, key: &str) -> Vec<String> {
        match self.get_value(key) {
            Some(Value::Array(a)) => a.get_all(),
            _ => vec![],
        }
    }

    pub fn is_array(&mut self, key: &str) -> bool {
        match self.get_value(key) {
            Some(Value::Array(_)) => true,
            _ => false,
        }
    }

    pub fn is_assoc(&mut self, key: &str) -> bool {
        match self.get_value(key) {
            Some(Value::AssocArray(_)) => true,
            _ => false,
        }
    }

    pub fn get_position_params(&self) -> Vec<String> {
        match self.position_parameters.last() {
            Some(v) => v[1..].to_vec(),
            _       => vec![],
        }
    }

    fn get_position_param_pos(&self, key: &str) -> Option<usize> {
        if ! (key.len() == 1 && "0" <= key && key <= "9") {
            return None;
        }

        let n = key.parse::<usize>().unwrap();
        let layer = self.position_parameters.len();
        match n < self.position_parameters[layer-1].len() {
            true  => Some(n),
            false => None,
        }
    }

    pub fn set_layer_param(&mut self, name: &str, val: &str, layer: usize) -> bool {
        match env::var(name) {
            Ok(_) => env::set_var(name, val),
            _     => {},
        }
        if self.parameters[layer].get(name).is_none() {
            self.parameters[layer].insert(name.to_string(), Variable::from(val));
            return true;
        }

        let mut v = self.parameters[layer].get(name).unwrap().clone();
        if v.attributes.contains('r') {
            return false;
        }

        v.value = match &v.value {
            Value::Special(sp) => {sp.get_data(&mut v.clone(), val); return true;},
            _ => Value::Single(SingleData::from(val)),
        };

        self.parameters[layer].insert(name.to_string(), v);

        true
    }

    pub fn set_param(&mut self, key: &str, val: &str) -> bool {
        self.set_layer_param(key, val, 0)
    }

    pub fn set_special_param(&mut self, key: &str, get: fn(&mut Variable)->Value,
                             set: Option<fn(&mut Variable, val: &str) -> String>) {
        let init = match set {
            Some(f) => f(&mut Variable::from(""), ""),
            None => "".to_string(),
        };

        self.parameters[0].insert(
            key.to_string(),
            Variable {
                value: Value::Special( SpecialData {
                    data: init,
                    attributes: "".to_string(),
                    dynamic_get: get,
                }),
                ..Default::default()
            }
        );        
    }

    pub fn set_local_param(&mut self, key: &str, val: &str) -> bool {
        let layer = self.parameters.len();
        self.set_layer_param(key, val, layer-1)
    }

    pub fn set_layer(&mut self, name: &str, v: Value, layer: usize) -> bool {
        self.parameters[layer].insert( name.to_string(), Variable::from(v));
        true
    }

    pub fn set_layer_assoc(&mut self, name: &str, layer: usize) -> bool {
        self.parameters[layer]
            .insert(name.to_string(), Variable::from(HashMap::new()));        
        true
    }

    pub fn set_layer_array_elem(&mut self, key: &str, val: &String, layer: usize, pos: usize) -> bool {
        match self.parameters[layer].get_mut(key) {
            Some(v) => v.set_array_elem(pos, val), 
            _ => return false,
        }
    }

    pub fn set_layer_assoc_elem(&mut self, name: &str, key: &String, val: &String, layer: usize) -> bool {
        match self.parameters[layer].get_mut(name) {
            Some(v) => v.set_assoc_elem(key, val), 
            _ => false,
        }
    }

    pub fn set_array_elem(&mut self, name: &str, val: &String, pos: usize) -> bool {
        self.set_layer_array_elem(name, val, 0, pos)
    }

    pub fn set_assoc_elem(&mut self, name: &str, key: &String, val: &String) -> bool {
        self.set_layer_assoc_elem(name, key, val, 0)
    }

    pub fn set_local_assoc_elem(&mut self, name: &str, key: &String, val: &String) -> bool {
        let layer = self.parameters.len();
        self.set_layer_assoc_elem(name, key, val, layer-1)
    }

    pub fn set(&mut self, name: &str, v: Value) -> bool {
        self.set_layer(name, v, 0)
    }

    pub fn set_local(&mut self, name: &str, v: Value) -> bool {
        let layer = self.parameters.len();
        self.set_layer(name, v, layer-1)
    }

    pub fn set_local_array_elem(&mut self, name: &str, val: &String, pos: usize) -> bool {
        let layer = self.parameters.len();
        self.set_layer_array_elem(name, val, layer-1, pos)
    }

    pub fn push_local(&mut self) { self.parameters.push(HashMap::new()); }
    pub fn pop_local(&mut self) { self.parameters.pop(); }
    pub fn get_layer_num(&mut self) -> usize { self.parameters.len() }

    pub fn get_keys(&mut self) -> Vec<String> {
        let mut keys = HashSet::new();
        for layer in &self.parameters {
            layer.keys().for_each(|k| {keys.insert(k);} );
        }
        let mut ans: Vec<String> = keys.iter().map(|c| c.to_string()).collect();
        ans.sort();
        ans
    }

    pub fn replace_alias(&mut self, word: &mut String) -> bool {
        let before = word.clone();
        match self.replace_alias_core(word) {
            true => {
                self.alias_memo.push( (before, word.clone()) );
                true
            },
            false => false,
        }
    }

    fn replace_alias_core(&self, word: &mut String) -> bool {
        if ! self.flags.contains('i') {
            return false;
        }

        let mut ans = false;
        let mut prev_head = "".to_string();

        loop {
            let head = match word.replace("\n", " ").split(' ').nth(0) {
                Some(h) => h.to_string(),
                _ => return ans,
            };

            if prev_head == head {
                return ans;
            }
    
            if let Some(value) = self.aliases.get(&head) {
                *word = word.replacen(&head, value, 1);
                ans = true;
            }
            prev_head = head;
        }
    }

    pub fn unset_var(&mut self, key: &str) {
        for layer in &mut self.parameters {
            layer.remove(key);
        }
    }

    pub fn unset_function(&mut self, key: &str) {
        self.functions.remove(key);
    }

    pub fn unset(&mut self, key: &str) {
        self.unset_var(key);
        self.unset_function(key);
    }
}
