//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::abst_elems::ListElem;
use crate::elem_arg_delimiter::ArgDelimiter;
use crate::Feeder;
use crate::elem_end_of_command::Eoc;

pub struct BlankPart {
    pub text: String,
}

impl ListElem for BlankPart {
    fn get_text(&self) -> String { self.text.clone() }
}

impl BlankPart {
    pub fn new() -> BlankPart{
        BlankPart {
            text: "".to_string(),
        }
    }

    pub fn parse(text: &mut Feeder) -> Option<BlankPart> {
        let mut ans = BlankPart::new();
    
        loop {
            if let Some(d) = ArgDelimiter::parse(text) {
                ans.text += &d.text;
            }
            else if let Some(e) = Eoc::parse(text) {
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
