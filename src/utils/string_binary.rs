//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::ffi::CString;
use std::io::{stdout, Write};

pub fn to_carg(arg: &str) -> CString {
    let mut tmp = String::new();
    let mut unicode8num = 0;

    for c in arg.chars() {
        if c as u32 >= 0xE080 && c as u32 <= 0xE0FF { //a char in [0x80, 0xFF] is shifted to an
            let num: u8 = (c as u32 - 0xE000) as u8;  // unused UTF-8 region.
            let ch = unsafe { String::from_utf8_unchecked(vec![num]) };
            tmp.push_str(&ch);
        } else if c as u32 >= 0xE200 && c as u32 <= 0xE4FF {
            unicode8num <<= 8;
            unicode8num += c as u32 & 0xFF;
        } else if c as u32 >= 0xE100 && c as u32 <= 0xE1FF {
            unicode8num <<= 8;
            unicode8num += c as u32 & 0xFF;
            let ch = unsafe { char::from_u32_unchecked(unicode8num) }.to_string();
            unicode8num = 0; //　^ An error occurs on debug mode.
            tmp.push_str(&ch);
        } else if c != '\0' { //Null chars are omitted since any command cannot handle args containing
            tmp.push(c);      //them. 
        }
    }
    CString::new(tmp.to_string()).unwrap()
}

pub fn to_stdout(arg: &str) {
    let mut unicode8num = 0;

    for c in arg.chars() {
        if c as u32 >= 0xE080 && c as u32 <= 0xE0FF { //a char in [0x80, 0xFF] is shifted to an
            let num: u8 = (c as u32 - 0xE000) as u8;  // unused UTF-8 region.
            let ch = unsafe { String::from_utf8_unchecked(vec![num]) };
            print!("{}", ch);
        } else if c as u32 >= 0xE200 && c as u32 <= 0xE4FF {
            unicode8num <<= 8;
            unicode8num += c as u32 & 0xFF;
        } else if c as u32 >= 0xE100 && c as u32 <= 0xE1FF {
            unicode8num <<= 8;
            unicode8num += c as u32 & 0xFF;
            let ch = unsafe { char::from_u32_unchecked(unicode8num) }.to_string();
            unicode8num = 0; //　^ An error occurs on debug mode.
            print!("{}", ch);
        } else {
            print!("{}", c);
        }
    }
    stdout().flush().unwrap()
}

pub fn to_cargs(args: &[String]) -> Vec<CString> {
    args.iter().map(|s| to_carg(s)).collect()
}
