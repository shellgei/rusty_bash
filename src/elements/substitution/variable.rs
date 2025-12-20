//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::database::data::uninit::Uninit;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::arg;
use crate::{Feeder, ShellCore};
use super::subscript::Subscript;

#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub text: String,
    pub name: String,
    pub index: Option<Subscript>,
    pub lineno: usize,
}

impl Variable {
    pub fn get_index(
        &mut self,
        core: &mut ShellCore,
        right_is_array: bool,
        append: bool,
    ) -> Result<Option<String>, ExecError> {
        if let Some(mut s) = self.index.clone() {
            if s.text == "[]" {
                return Err(ExecError::ArrayIndexInvalid("".to_string()));
            }
            if s.text.chars().all(|c| " \n\t[]".contains(c)) {
                if core.db.is_assoc(&self.name) {
                    let mut index = s.text.clone();
                    index.remove(0);
                    index.pop();
                    return Ok(Some(index));
                }
                return Ok(Some("0".to_string()));
            }
            let index = s.eval(core, &self.name)?;
            return Ok(Some(index));
        }

        if core.db.is_array(&self.name) && !append && !right_is_array {
            Ok(Some("0".to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn is_array(&mut self) -> bool {
        self.is_pos_param_array() || self.is_var_array()
    }

    pub fn is_pos_param_array(&mut self) -> bool {
        self.name == "@" || self.name == "*"
    }

    pub fn is_var_array(&mut self) -> bool {
        if self.index.is_none() {
            return false;
        }
        let sub = &self.index.as_ref().unwrap().text;
        sub == "[*]" || sub == "[@]"
    }

    pub fn set_value(&mut self, value: &str, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.index.is_none() {
            return core.db.set_param(&self.name, value, None);
        }

        let index = self.index.clone().unwrap().eval(core, &self.name)?;
        core.db.set_param2(&self.name, &index, value, None)
    }

    pub fn parse_and_set(arg: &str, value: &str, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut f = Feeder::new(arg);
        match Self::parse(&mut f, core)? {
            Some(mut v) => {
                if !f.is_empty() {
                    return Err(ExecError::InvalidName(arg.to_string()));
                }
                v.set_value(value, core)
            }
            None => Err(ExecError::InvalidName(arg.to_string())),
        }
    }

    pub fn init_variable(
        &self,
        core: &mut ShellCore,
        layer: Option<usize>,
        args: &mut Vec<String>,
    ) -> Result<(), ExecError> {
        let mut prev = vec![];

        let exists_in_layer = if let Some(l) = layer {
            core.db.exist_l(&self.name, l)
        } else {
            false
        };
        if (layer.is_none() && core.db.exist(&self.name)) || exists_in_layer {
            prev = vec![core.db.get_param(&self.name)?];
        }

        let i_opt = arg::consume_arg("-i", args);
        let a_opt = arg::consume_arg("-a", args);
        let la_opt = arg::consume_arg("-A", args);

        if a_opt || (!la_opt && self.index.is_some()) {
            let data = match prev.is_empty() {
                true  => None,
                false => Some(prev),
            };
            return core.db.init_array(&self.name, data, layer, i_opt);
            //TODO: ^ Maybe, there is a case where an assoc must be
            //prepared.
        } else if la_opt {
            core.db.init_assoc(&self.name, layer, false, i_opt)?;
            if !prev.is_empty() {
                core.db.set_assoc_elem(&self.name, "0", &prev[0], layer)?;
            }
            return Ok(());
        }

        match prev.len() {
            0 => {
                match i_opt {
                    true =>  core.db.init_as_num(&self.name, "", layer),
                    false => {
                        let mut opts = String::new();
                        if a_opt {
                            opts.push('a');
                        }
                        if la_opt {
                            opts.push('A');
                        }
                        let d = Box::new(Uninit::new(&opts));
                        core.db.set_entry(layer.unwrap_or(0), &self.name, d)
                    },
                }
            },
            _ => {
                match i_opt {
                    true => core.db.init_as_num(&self.name, &prev[0], layer),
                    false => core.db.set_param(&self.name, &prev[0], layer),
                }
            },
        }
    }

    pub fn exist(&self, core: &mut ShellCore) -> Result<bool, ExecError> {
        //used in value_check.rs
        if core.db.is_array(&self.name) || core.db.is_assoc(&self.name) {
            if core.db.get_vec(&self.name, false)?.is_empty() {
                return Ok(false);
            }

            if self.index.is_none() {
                return core.db.has_key(&self.name, "0");
            }
        }

        if let Some(sub) = self.index.clone().as_mut() {
            let index = sub.eval(core, &self.name)?;
            return core.db.has_key(&self.name, &index);
        }

        Ok(core.db.exist(&self.name))
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return Ok(None);
        }

        let mut ans = Self {
            lineno: feeder.lineno,
            ..Default::default()
        };

        let name = feeder.consume(len);
        ans.name = name.clone();
        ans.text += &name;

        if let Some(s) = Subscript::parse(feeder, core)? {
            ans.text += &s.text.clone();
            ans.index = Some(s);
        };

        Ok(Some(ans))
    }
}
