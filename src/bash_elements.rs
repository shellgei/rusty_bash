//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

trait Element {
    fn info(&self);
}

struct Core {
    elems: Vec<Box<dyn Element>>,
    text: String,
    text_pos: u32
}

struct Comment {
    core: Core
}

impl Element for Comment {
    fn info(&self){
        println!("({}[byte] text)", self.core.text_pos);
        println!("{}", self.core.text);
    }
}
