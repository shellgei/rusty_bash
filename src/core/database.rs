//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

mod data;

use self::data::Data;
use self::data::random::RandomVar;
use self::data::seconds::Seconds;
use self::data::srandom::SRandomVar;
use self::data::single::SingleData;
use self::data::single_ondemand::OnDemandSingle;
use crate::utils::clock;
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
        ans.params[0].insert("SECONDS".to_string(), Box::new(Seconds::new()));
        ans.params[0].insert("EPOCHSECONDS".to_string(),
                             Box::new(OnDemandSingle::new(clock::get_epochseconds)));
        ans.params[0].insert("EPOCHREALTIME".to_string(),
                             Box::new(OnDemandSingle::new(clock::get_epochrealtime)));
        ans
    }

    pub fn get_param(&mut self, name: &str) -> Result<String, ExecError> {
        if let Ok(n) = name.parse::<usize>() {
            let scope = &self.position_parameters.last().unwrap();
            if  n < scope.len() {
                return Ok(scope[n].to_string());
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

    pub fn solve_set_scope(&mut self, name: &str,
                           scope: Option<usize>) -> usize {
        if let Some(ly) = scope {
            return ly;
        }

        self.get_scope_pos(name).unwrap_or(0)
    }

    pub fn get_param_keys(&mut self) -> Vec<String> {
        let mut keys = HashSet::new();
        for scope in &self.params {
            scope.keys()
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
        if let Some(scope) = self.get_scope_pos(name) {
            if let Some(d) = self.params[scope].get_mut(name) {
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

    pub fn get_scope_pos(&mut self, name: &str) -> Option<usize> {
        let num = self.params.len();
        (0..num)
            .rev()
            .find(|&scope| self.params[scope].contains_key(name))
    }

    pub fn set_param(&mut self, name: &str, val: &str,
                     scope: Option<usize>) -> Result<(), ExecError> {
        let scope = self.solve_set_scope(name, scope);

        if let Some(d) = self.params[scope].get_mut(name) {
            d.set_as_single(name, val)?;
        }else{
            self.params[scope].insert(name.to_string(),
                                      Box::new(SingleData::from(val)));
        }

        if scope == 0 && env::var(name).is_ok() {
            unsafe{env::set_var(name, val)};
        }
        Ok(())
    }

    pub fn get_param_scope_ref(&mut self, scope: usize) -> &mut HashMap<String, Box::<dyn Data>> {
        &mut self.params[scope]
    }

    pub fn push_local(&mut self) {
        self.params.push(HashMap::new());
    }

    pub fn pop_local(&mut self) {
        self.params.pop();
    }

    pub fn get_scope_num(&mut self) -> usize {
        self.params.len()
    }

    pub fn set_flag(&mut self, name: &str, flag: char, scope: usize) {
        if let Some(d) = self.params[scope].get_mut(name) {
            d.set_flag(flag);
        }
    }

    pub fn get_flags(&mut self, name: &str) -> &str {
        match self.get_scope_pos(name) {
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
        for scope in (0..num).rev() {
            if self.unset_var_scope(scope, name)? {
                return Ok(true)
            }
        }
        Ok(false)
    }

    fn unset_var_scope(&mut self, scope: usize, name: &str)
                                  -> Result<bool, ExecError> {
        if ! self.params[scope].contains_key(name) {
            return Ok(false)
        }
        self.params[scope].get_mut(name)
                          .unwrap()
                          .readonly_check(name)?;

        self.params[scope].remove(name);

        if scope == 0 {
            unsafe{env::remove_var(name)};
        }
        Ok(true)
    }
}
