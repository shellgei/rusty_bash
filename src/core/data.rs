//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::elements::array::Array;
use crate::elements::word::Word;
use crate::elements::command::function_def::FunctionDefinition;
use std::env;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Single(Word),
    EvaluatedSingle(String),
    Array(Array),
    EvaluatedArray(Vec<String>),
}

#[derive(Debug)]
pub struct Data {
    pub flags: String,
    parameters: Vec<HashMap<String, Value>>,
    pub position_parameters: Vec<Vec<String>>,
    pub aliases: HashMap<String, String>,
    pub functions: HashMap<String, FunctionDefinition>,
    pub alias_memo: Vec<(String, String)>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            flags: String::new(),
            parameters: vec![HashMap::new()],
            position_parameters: vec![vec![]],
            aliases: HashMap::new(),
            functions: HashMap::new(),
            alias_memo: vec![],
        }
    }

    pub fn get_param(&mut self, key: &str) -> String {
        if key == "-" {
            return self.flags.clone();
        }

        if key == "@" || key == "*" {
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
                if pos == "@" {
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
            match self.parameters[layer].get(key) {
                Some(v) => return Some(v.clone()),
                _ => {},
            }
        }
        None
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

    pub fn set_layer_param(&mut self, key: &str, val: &str, layer: usize) {
        match env::var(key) {
            Ok(_) => env::set_var(key, val),
            _     => {},
        }

        self.parameters[layer].insert(key.to_string(), Value::EvaluatedSingle(val.to_string()));
    }

    pub fn set_param(&mut self, key: &str, val: &str) {
        self.set_layer_param(key, val, 0);
    }

    pub fn unset(&mut self, key: &str) {
        for layer in &mut self.parameters {
            layer.remove(key);
        }
    }

    pub fn set_local_param(&mut self, key: &str, val: &str) {
        let layer = self.parameters.len();
        self.set_layer_param(key, val, layer-1);
    }

    pub fn set_layer_array(&mut self, key: &str, vals: &Vec<String>, layer: usize) {
        self.parameters[layer].insert(key.to_string(), Value::EvaluatedArray(vals.to_vec()));
    }

    pub fn set_array(&mut self, key: &str, vals: &Vec<String>) {
        self.set_layer_array(key, vals, 0);
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
}
