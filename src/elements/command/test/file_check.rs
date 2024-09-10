//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use crate::elements::command::test::Elem;

pub fn is_file(name: &String, stack: &mut Vec<Elem>) -> Result<(), String> {
    let ans = Path::new(name).is_file();
    stack.push( Elem::Ans(ans) );
    Ok(())
}

pub fn is_block(name: &String, stack: &mut Vec<Elem>) -> Result<(), String> {
    let meta = match fs::metadata(name) {
        Ok(m) => m,
        _  => {
            stack.push( Elem::Ans(false) );
            return Ok(());
        },
    };
    let ans = meta.file_type().is_block_device();
    stack.push( Elem::Ans(ans) );
    Ok(())
}

pub fn is_char(name: &String, stack: &mut Vec<Elem>) -> Result<(), String> {
    let meta = match fs::metadata(name) {
        Ok(m) => m,
        _  => {
            stack.push( Elem::Ans(false) );
            return Ok(());
        },
    };
    let ans = meta.file_type().is_char_device();
    stack.push( Elem::Ans(ans) );
    Ok(())
}

