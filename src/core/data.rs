//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::elements::array::Array;
use crate::elements::word::Word;
use crate::elements::command::function_def::FunctionDefinition;
use std::{env, process};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub enum Value {
    #[default]
    None,
    Single(Word),
    EvaluatedSingle(String),
    Array(Array),
    AssocArray(HashMap::<String, String>),
    EvaluatedArray(Vec<String>),
}

#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub value: Value,
    attributes: String,
    pub dynamic_get: Option<fn(&mut Variable) -> Value>,
    pub dynamic_set: Option<fn(&mut Variable, &str) -> Value>,
}

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

    pub fn get_param(&mut self, key: &str) -> String {
        if key == "-" {
            return self.flags.clone();
        }

        if key == "@" || key == "*" { // $@ should return an array in a double quoted
                                      // subword. Therefore another access method is used there. 
            return match self.position_parameters.last() {
                Some(a) => a[1..].join(" "),
                _       => "".to_string(),
            };
        }

        if let Some(n) = self.get_position_param_pos(key) {
            let layer = self.position_parameters.len();
            return self.position_parameters[layer-1][n].to_string();
        }

        match self.get_value(key) {
            Some(Value::EvaluatedSingle(v)) => return v.to_string(),
            Some(Value::EvaluatedArray(a)) => {
                match a.len() {
                    0 => return "".to_string(),
                    _ => return a[0].to_string(),
                }
            },
            _  => {},
        }

        match env::var(key) {
            Ok(v) => {
                self.set_layer_param(key, &v, 0);
                v
            },
            _ => "".to_string()
        }
    }

    pub fn get_array(&mut self, key: &str, pos: &str) -> String {
        match self.get_value(key) {
            Some(Value::EvaluatedArray(a)) => {
                if pos == "@" || pos == "*" {
                    return a.join(" ");
                } else if let Ok(n) = pos.parse::<usize>() {
                    if n < a.len() {
                        return a[n].clone();
                    }
                }
            },
            Some(Value::EvaluatedSingle(v)) => {
                match pos.parse::<usize>() {
                    Ok(0) => return v.to_string(),
                    Ok(_) => return "".to_string(),
                    _ => return v.to_string(), 
                }
            },
            _ => {},
        }
        "".to_string()
    }

    pub fn get_value(&mut self, key: &str) -> Option<Value> {
        let num = self.parameters.len();
        for layer in (0..num).rev()  {
            match self.parameters[layer].get_mut(key) {
                Some(v) => { 
                    if let Some(f) = v.dynamic_get {
                        return Some(f(v).clone())
                    } else {
                        return Some(v.value.clone())
                    }
                },
                _ => {},
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

    pub fn get_array_len(&mut self, key: &str) -> usize {
        match self.get_value(key) {
            Some(Value::EvaluatedArray(a)) => a.len(),
            _ => 0,
        }
    }

    pub fn get_array_all(&mut self, key: &str) -> Vec<String> {
        match self.get_value(key) {
            Some(Value::EvaluatedArray(a)) => a.clone(),
            _ => vec![],
        }
    }

    pub fn is_array(&mut self, key: &str) -> bool {
        match self.get_value(key) {
            Some(Value::EvaluatedArray(_)) => true,
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
        if n < self.position_parameters[layer-1].len() {
            Some(n)
        }else{
            None
        }
    }

    pub fn set_layer_param(&mut self, key: &str, val: &str, layer: usize) -> bool {
        match env::var(key) {
            Ok(_) => env::set_var(key, val),
            _     => {},
        }

        self.parameters[layer].entry(key.to_string())
        .and_modify(|v| {
            if v.attributes.contains('r') {
                // error : "readonly variable"
            } else {
                v.value = match v.dynamic_set {
                    Some(f) => f(v, val),
                    None    => Value::EvaluatedSingle(val.to_string()),
                };
            }
        })
        .or_insert(
            Variable {
                value: Value::EvaluatedSingle(val.to_string()),
                ..Default::default()
            }
        );
        true
    }

    pub fn set_param(&mut self, key: &str, val: &str) -> bool {
        self.set_layer_param(key, val, 0)
    }

    pub fn set_special_param(&mut self, key: &str, get: fn(&mut Variable)->Value,
                             set: Option<fn(&mut Variable, val: &str)->Value>) {
        self.parameters[0].insert(
            key.to_string(),
            Variable {
                value: Value::None,
                dynamic_get: Some(get),
                dynamic_set: set,
                ..Default::default()
            }
        );        
    }

    pub fn set_local_param(&mut self, key: &str, val: &str) {
        let layer = self.parameters.len();
        self.set_layer_param(key, val, layer-1);
    }

    pub fn set_layer_array(&mut self, key: &str, vals: &Vec<String>, layer: usize) -> bool {
        self.parameters[layer].insert(
            key.to_string(),
            Variable {
                value: Value::EvaluatedArray(vals.to_vec()),
                ..Default::default()
            }
        );        
        true
    }

    pub fn set_layer_assoc(&mut self, key: &str, layer: usize) -> bool {
        self.parameters[layer].insert(
            key.to_string(),
            Variable {
                value: Value::AssocArray(HashMap::new()),
                ..Default::default()
            }
        );        
        true
    }

    pub fn set_layer_array_elem(&mut self, key: &str, val: &String, layer: usize, pos: usize) -> bool {
        let value = match self.parameters[layer].get_mut(key) {
            Some(v) => v, 
            _ => return false,
        };
        let array = match &mut value.value {
            Value::EvaluatedArray(a) => a, 
            _ => return false,
        };

        if array.len() > pos {
            array[pos] = val.clone();
        }
        true
    }

    pub fn set_layer_assoc_elem(&mut self, key: &str, val: &String, layer: usize, pos: &String) -> bool {
        let value = match self.parameters[layer].get_mut(key) {
            Some(v) => v, 
            _ => return false,
        };
        let array = match &mut value.value {
            Value::AssocArray(a) => a, 
            _ => return false,
        };

        array.insert(pos.to_string(), val.to_string());
        true
    }

    pub fn set_array(&mut self, key: &str, vals: &Vec<String>) -> bool {
        self.set_layer_array(key, vals, 0);
        true
    }

    pub fn set_assoc(&mut self, key: &str) -> bool {
        self.set_layer_assoc(key, 0);
        true
    }

    pub fn set_array_elem(&mut self, key: &str, val: &String, pos: usize) -> bool {
        self.set_layer_array_elem(key, val, 0, pos)
    }

    pub fn set_assoc_elem(&mut self, key: &str, val: &String, pos: &String) -> bool {
        self.set_layer_assoc_elem(key, val, 0, pos)
    }

    pub fn set_local_array(&mut self, key: &str, vals: &Vec<String>) {
        let layer = self.parameters.len();
        self.set_layer_array(key, vals, layer-1);
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

    pub fn get_keys(&mut self) -> Vec<String> {
        let mut output = HashSet::new();
        for layer in &self.parameters {
            for k in layer.keys() {
                output.insert(k);
            }
        }
        let mut ans: Vec<String> = output.iter().map(|c| c.to_string()).collect();
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
        if self.flags.find('i') == None {
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

    pub fn not_set(v: &mut Variable, _var: &str) -> Value {
        v.value.clone()
    }
    
}
