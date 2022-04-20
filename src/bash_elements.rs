//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::ffi::CString;

pub struct Core {
    elems: Vec<Box<dyn Element>>,
    text: String,
    text_pos: u32
}

impl Core {
    fn info(&self){
        println!("({}[byte] text)", self.text_pos);
        println!("{}", self.text);
    }

    pub fn new() -> Core{
        Core{
            elems: Vec::new(),
            text: "".to_string(),
            text_pos: 0
        }
    }
}

trait Element {
    fn info(&self);
}

/*
struct Comment {
    core: Core
}
*/

pub struct CommandWithArgs {
    pub core: Core,
    pub args: Box<[CString]>
}

impl Element for CommandWithArgs {
    fn info(&self){
        self.core.info();
    }
}
