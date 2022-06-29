//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::elem_end_of_command::Eoc;
use crate::scanner::scanner_while;

pub struct BlankPart {
}

impl BlankPart {
    pub fn parse(text: &mut Feeder) -> Option<String> {
        let mut ans = String::new();
    
        loop {
            let d = scanner_while(text, 0, " \t");
            ans += &text.consume(d);

            if let Some(e) = Eoc::parse(text) {
                ans += &e.text;
            }
            else{break;};
        };
    
        if ans.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
