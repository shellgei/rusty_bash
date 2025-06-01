//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use std::collections::HashSet;
use super::DataBase;
use super::data::Data;

impl DataBase {
    pub fn get_ref(&mut self, name: &str) -> Option<&mut Box<dyn Data>> {
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

    pub fn get_layer_pos(&mut self, name: &str) -> Option<usize> {
        let num = self.params.len();
        for layer in (0..num).rev()  {
            if self.params[layer].get(name).is_some() {
                return Some(layer);
            }
        }
        None
    }

    pub fn get_position_params(&self) -> Vec<String> {
        match self.position_parameters.last() {
            Some(v) => v[1..].to_vec(),
            _       => vec![],
        }
    }
}

pub fn special_param(db :&DataBase, name: &str) -> Option<String> {
    let val = match name {
        "-" => db.flags.clone(),
        "?" => db.exit_status.to_string(),
        "_" => db.last_arg.clone(),
        "#" => {
            let pos = db.position_parameters.len() - 1;
            (db.position_parameters[pos].len() - 1).to_string()
        },
        _ => return None,
    };

    Some(val)
}

pub fn connected_position_params(db :&mut DataBase, aster: bool) -> Result<String, ExecError> {
    let mut joint = " ".to_string();
    if aster {
        joint = db.get_ifs_head();
    }

    match db.position_parameters.last() {
        Some(a) => Ok(a[1..].join(&joint)),
        _       => Ok("".to_string()),
    }
}

pub fn position_param(db: &DataBase, pos: usize) -> Result<String, ExecError> {
    let layer = db.position_parameters.len();
    return match db.position_parameters[layer-1].len() > pos {
        true  => Ok(db.position_parameters[layer-1][pos].to_string()),
        false => Ok(String::new()),
    };
}

/*
pub fn array_elem(db: &mut DataBase, name: &str, pos: &str) -> Result<String, ExecError> {
    let layer = match db.get_layer_pos(name) {
        Some(n) => n,
        _ => return Ok("".to_string()),
    };

    let ifs = db.get_ifs_head();
    db.params[layer].get_mut(name).unwrap().get_as_array_or_assoc(pos, &ifs)
}*/
