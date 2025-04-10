//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use crate::core::DataBase;

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

pub fn connected_position_params(db :&mut DataBase) -> Result<String, ExecError> {
    let ifs = db.get_param("IFS").unwrap_or(" \t\n".to_string());
    let joint = match ifs.chars().nth(0) {
        None => "".to_string(),
        Some(c) => c.to_string(),
    };

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

pub fn array_elem(db: &mut DataBase, name: &str, pos: &str) -> Result<String, ExecError> {
    let layer = match db.get_layer_pos(name) {
        Some(n) => n,
        _ => return Ok("".to_string()),
    };

    db.params[layer].get_mut(name).unwrap().get_as_array_or_assoc(pos)
}
