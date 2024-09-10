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

fn file_type_check(name: &String, stack: &mut Vec<Elem>, tp: &str) -> Result<(), String> {
    let meta = match fs::metadata(name) {
        Ok(m) => m,
        _  => {
            stack.push( Elem::Ans(false) );
            return Ok(());
        },
    };
    let ans = match tp {
        "-b" => meta.file_type().is_block_device(),
        "-c" => meta.file_type().is_char_device(),
        _ => false,
    };

    stack.push( Elem::Ans(ans) );
    Ok(())
}

pub fn is_block(name: &String, stack: &mut Vec<Elem>) -> Result<(), String> {
    file_type_check(name, stack, "-b")
        /*
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
        */
}

pub fn is_char(name: &String, stack: &mut Vec<Elem>) -> Result<(), String> {
    file_type_check(name, stack, "-c")
    /*
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
    */
}

