//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

trait Element {
    fn info(&self);
}

struct Comment {
    text: String
}

impl Element for Comment {
    fn info(&self){
        println!("{}", self.text);
    }
}
