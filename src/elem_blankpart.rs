//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::abst_hand_input_unit::HandInputUnit;
use crate::elem_arg_delimiter::ArgDelimiter;
use crate::CommandElem;
use crate::Feeder;
use crate::elem_end_of_command::Eoc;

pub struct BlankPart {
    pub elems: Vec<Box<dyn CommandElem>>,
    text: String,
}

impl HandInputUnit for BlankPart {
}

impl BlankPart {
    pub fn new() -> BlankPart{
        BlankPart {
            elems: vec!(),
            text: "".to_string(),
        }
    }

    pub fn push(&mut self, s: Box<dyn CommandElem>){
        self.text += &s.text();
        self.elems.push(s);
    }

    pub fn return_if_valid(ans: BlankPart) -> Option<BlankPart> {
        if ans.elems.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }

    pub fn parse(text: &mut Feeder) -> Option<BlankPart> {
        let mut ans = BlankPart::new();
    
        loop {
            if let Some(d) = ArgDelimiter::parse(text)          {ans.push(Box::new(d));}
            else if let Some(e) = Eoc::parse(text){ans.push(Box::new(e));}
            else{break;};
        };
    
        BlankPart::return_if_valid(ans)
    }
}
