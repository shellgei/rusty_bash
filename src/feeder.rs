//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub struct Feeder {
    remaining: String,
}

impl Feeder {
    pub fn new() -> Feeder {
        Feeder {
            remaining: String::new(),
        }
    }
}
