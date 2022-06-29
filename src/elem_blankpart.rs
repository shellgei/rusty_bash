//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::elem_end_of_command::Eoc;
use crate::scanner::scanner_while;

pub struct BlankPart {
    text: String,
}

impl BlankPart {
    pub fn get_text(&self) -> String { self.text.clone() }

    pub fn parse(text: &mut Feeder) -> Option<BlankPart> {
        let mut ans = BlankPart { text: String::new() };
    
        loop {
            let d = scanner_while(text, 0, " \t");
            ans.text += &text.consume(d);

            if let Some(e) = Eoc::parse(text) {
                ans.text += &e.text;
            }
            else{break;};
        };
    
        if ans.text.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
