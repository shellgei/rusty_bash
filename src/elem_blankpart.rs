//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

/*
use crate::Feeder;
use crate::scanner::*;

pub struct BlankPart {
}

impl BlankPart {
    pub fn parse(text: &mut Feeder) -> Option<String> {
        let mut ans = String::new();
    
        loop {
            let d = scanner_while(text, 0, " \t");
            ans += &text.consume(d);

            if text.len() == 0 {
                break;
            }

            let pos = scanner_end_of_com(text, 0);
            if pos != 0 {
                ans += &text.consume(pos);
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
*/
