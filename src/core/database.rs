//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

mod data;

use crate::exit;
use crate::elements::command::function_def::FunctionDefinition;
use std::{env, process};
use std::collections::{HashMap, HashSet};
use crate::utils::{random, clock, error};
use self::data::Data;
use self::data::assoc::AssocData;
use self::data::single::SingleData;
use self::data::array::ArrayData;
use self::data::special::SpecialData;

#[derive(Debug, Default)]
pub struct DataBase {
    pub flags: String,
    params: Vec<HashMap<String, Box<dyn Data>>>,
    param_options: Vec<HashMap<String, String>>,
    pub position_parameters: Vec<Vec<String>>,
    pub aliases: HashMap<String, String>,
    pub functions: HashMap<String, FunctionDefinition>,
    pub alias_memo: Vec<(String, String)>,
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

        data.set_param("$", &process::id().to_string());
        data.set_param("BASHPID", &process::id().to_string());
        data.set_param("BASH_SUBSHELL", "0");
        data.set_param("?", "0");
        data.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()));

        data.set_special_param("SRANDOM", random::get_srandom);
        data.set_special_param("RANDOM", random::get_random);
        data.set_special_param("EPOCHSECONDS", clock::get_epochseconds);
        data.set_special_param("EPOCHREALTIME", clock::get_epochrealtime);
        data.set_special_param("SECONDS", clock::get_seconds);

        data.call_speial("SECONDS");

        data.set_array("FUNCNAME", vec![]);

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

        if let Some(ans) = self.call_speial(name) {
            return ans;
        }

        match self.get_clone(name).as_mut() {
            Some(d) => {
                if d.is_array() {
                    return match d.get_as_array("0") {
                        Some(s) => s,
                        None => "".to_string(),
                    }
                }
                if d.is_single() {
                    return match d.get_as_single() {
                        Some(s) => s,
                        None => "".to_string(),
                    }
                }
            },
            _ => {},
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
        match self.get_clone(name).as_mut() {
            Some(d) => {
                if d.is_assoc() {
                    if let Some(ans) = d.get_as_assoc(pos) {
                        return ans;
                    }
                }
                if d.is_array() {
                    if let Some(ans) = d.get_as_array(pos) {
                        return ans;
                    }
                }
            },
            None => {},
        }

        "".to_string()
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

    fn call_speial(&mut self, name: &str) -> Option<String> {
        let num = self.params.len();
        for layer in (0..num).rev()  {
            if let Some(v) = self.params[layer].get_mut(name) {
                if v.is_special() {
                    return v.get_as_single();
                }
            }
        }
        None
    }

    fn get_clone(&mut self, name: &str) -> Option<Box<dyn Data>> {
        let num = self.params.len();
        for layer in (0..num).rev()  {
            if let Some(v) = self.params[layer].get_mut(name) {
                return Some(v.clone());
            }
        }
        None
    }

    pub fn has_value(&mut self, name: &str) -> bool {
        let num = self.params.len();
        for layer in (0..num).rev()  {
            if let Some(_) = self.params[layer].get(name) {
                return true;
            }
        }
        false
    }

    pub fn len(&mut self, key: &str) -> usize {
        match self.get_clone(key).as_mut() {
            Some(d) => d.len(),
            _ => 0,
        }
    }

    pub fn get_array_all(&mut self, key: &str) -> Vec<String> {
        match self.get_clone(key).as_mut() {
            Some(d) => {
                match d.get_all_as_array() {
                    Some(v) => v,
                    None => vec![],
                }
            },
            None => vec![],
        }
    }

    pub fn is_array(&mut self, name: &str) -> bool {
        match self.get_clone(name).as_mut() {
            Some(d) => return d.is_array(),
            _ => false,
        }
    }

    pub fn is_assoc(&mut self, key: &str) -> bool {
        match self.get_clone(key) {
            Some(d) => d.is_assoc(),
            None => false,
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

    fn has_flag(&mut self, name: &str, flag: char) -> bool {
        let layer = self.param_options.len() - 1;
        match self.param_options[layer].get(name) {
            None => false,
            Some(e) => e.contains(flag),
        }
    }

    pub fn set_layer_param(&mut self, name: &str, val: &str, layer: usize) -> Result<(), String> {
        if self.has_flag(name, 'r') {
            self.set_param("?", "1");
            return Err(error::readonly(name));
        }

        match env::var(name) {
            Ok(_) => env::set_var(name, val),
            _     => {},
        }
        match self.params[layer].get_mut(name) {
            Some(d) => {
                if d.is_single() {
                    return d.set_as_single(val);
                }
            },
            None => {
                self.params[layer].insert(name.to_string(), Box::new(SingleData::from(val)));
                return Ok(());
            },
        }
        Ok(())
    }

    pub fn set_param(&mut self, name: &str, val: &str) -> bool {
        self.set_layer_param(name, val, 0).is_ok() // TODO: return Result<(), String>
    }

    pub fn set_special_param(&mut self, key: &str, f: fn(&mut Vec<String>)-> String) {
        self.params[0].insert( key.to_string(), Box::new(SpecialData::from(f)) );
    }

    pub fn set_layer_array(&mut self, name: &str, v: Vec<String>, layer: usize) -> bool {
        if self.has_flag(name, 'r') {
            self.set_param("?", "1");
            let msg = error::readonly(name);
            eprintln!("{}", &msg);
            return false;
        }

        self.params[layer].insert( name.to_string(), Box::new(ArrayData::from(v)));
        true
    }

    pub fn set_layer_assoc(&mut self, name: &str, layer: usize) -> bool {
        if self.has_flag(name, 'r') {
            self.set_param("?", "1");
            let msg = error::readonly(name);
            eprintln!("{}", &msg);
            return false;
        }

        self.params[layer].insert(name.to_string(), Box::new(AssocData::default()));
        true
    }

    pub fn set_layer_array_elem(&mut self, name: &str, val: &String, layer: usize, pos: usize) -> bool {
        if self.has_flag(name, 'r') {
            self.set_param("?", "1");
            let msg = error::readonly(name);
            eprintln!("{}", &msg);
            return false;
        }

        match self.params[layer].get_mut(name) {
            Some(d) => d.set_as_array(&pos.to_string(), val),
            None    => {
                self.set_layer_array(name, vec![], layer);
                self.set_layer_array_elem(name, val, layer, pos)
            },
        }
    }

    pub fn set_layer_assoc_elem(&mut self, name: &str, key: &String, val: &String, layer: usize) -> bool {
        if self.has_flag(name, 'r') {
            self.set_param("?", "1");
            let msg = error::readonly(name);
            eprintln!("{}", &msg);
            return false;
        }

        match self.params[layer].get_mut(name) {
            Some(v) => v.set_as_assoc(key, val), 
            _ => false,
        }
    }

    pub fn set_array_elem(&mut self, name: &str, val: &String, pos: usize) -> bool {
        self.set_layer_array_elem(name, val, 0, pos)
    }

    pub fn set_assoc_elem(&mut self, name: &str, key: &String, val: &String) -> bool {
        self.set_layer_assoc_elem(name, key, val, 0)
    }

    pub fn set_array(&mut self, name: &str, v: Vec<String>) -> bool {
        self.set_layer_array(name, v, 0)
    }

    pub fn set_assoc(&mut self, name: &str) -> bool {
        self.set_layer_assoc(name, 0)
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

    pub fn unset_var(&mut self, name: &str) {
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

    pub fn set_flag(&mut self, name: &str, flag: char) {
        let layer = self.position_parameters.len() - 1;
        let rf = &mut self.param_options[layer];
        match rf.get_mut(name) {
            Some(d) => d.push(flag),
            None => {rf.insert(name.to_string(), "r".to_string()); },
        }
    }

    pub fn print(&mut self, name: &str) {
        if let Some(d) = self.get_clone(name) {
            d.print_with_name(name);
        }
    }
}
