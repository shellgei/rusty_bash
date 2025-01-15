//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::ExecError;
use super::ArithElem;
use crate::elements::subscript::Subscript;

pub fn to_operand(name: &String, sub: &mut Subscript, pre_increment: i64, post_increment: i64,
                   core: &mut ShellCore) -> Result<ArithElem, ExecError> {
    let key = sub.eval(core, name)?;

    let mut value_str = core.db.get_array_elem(name, &key)?;
    if value_str == "" {
        value_str = "0".to_string();
    }

    let mut value_num = match value_str.parse::<i64>() {
        Ok(n) => n,
        Err(_) => return Err(ExecError::Other(format!("{}: not an interger", &name))),
    };

    if pre_increment != 0 {
        value_num += pre_increment;
        set_value(name, &key, value_num, core)?;
        /*
        match set_value(name, &key, value_num, core) {
            Ok(()) => {},
            Err(e) => return Err(ExecError::Other(e)),
        }*/
    }

    let ans = Ok( ArithElem::Integer(value_num) );

    if post_increment != 0 {
        value_num += post_increment;
        set_value(name, &key, value_num, core)?;
        /*
        match set_value(name, &key, value_num, core) {
            Ok(()) => {},
            Err(e) => return Err(ExecError::Other(e)),
        }*/
    }
    ans
}

fn set_value(name: &String, key: &String, new_value: i64,
                     core: &mut ShellCore) -> Result<(), ExecError> {
    if let Ok(n) = key.parse::<i64>() {
        return match n >= 0 {
            true  => core.db.set_array_elem(name, &(new_value.to_string()), n as usize, None),
            false => Err(ExecError::Other("negative index".to_string())),
        };
    }

    core.db.set_assoc_elem(name, &(new_value.to_string()), key, None)
}

