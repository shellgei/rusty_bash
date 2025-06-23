//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::ffi::CString;

pub fn to_carg(arg: &String) -> CString {
    let mut tmp = String::new();
    let mut unicode8num = 0;

    for c in arg.chars() {
        if c as u32 >= 0xE080 && c as u32 <= 0xE0FF {
            let num: u8 = (c as u32 - 0xE000) as u8;
            let ch = unsafe{ String::from_utf8_unchecked(vec![num.try_into().unwrap()]) };
            tmp.push_str(&ch);
        }else if c as u32 >= 0xE200 && c as u32 <= 0xE4FF {
            unicode8num <<= 8;
            unicode8num += c as u32 & 0xFF;
        }else if c as u32 >= 0xE100 && c as u32 <= 0xE1FF {
            unicode8num <<= 8;
            unicode8num += c as u32 & 0xFF;
            let ch = unsafe { char::from_u32_unchecked(unicode8num) }.to_string();
            unicode8num = 0;  //　^ An error occurs on debug mode. 
            tmp.push_str(&ch);
        }else{
            tmp.push(c);
        }
    }
    CString::new(tmp.to_string()).unwrap()
}

pub fn to_cargs(args: &Vec<String>) -> Vec<CString> {
    args.iter().map(|a| to_carg(a)).collect()
}
