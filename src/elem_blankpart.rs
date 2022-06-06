//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elem_command::Executable;
use crate::ElemOfCommand;
use crate::Feeder;
use crate::parser::delimiter;
use crate::parser::end_of_command;

pub struct BlankPart {
    pub elems: Vec<Box<dyn ElemOfCommand>>,
    text: String,
}

impl Executable for BlankPart {
}

impl BlankPart {
    pub fn new() -> BlankPart{
        BlankPart {
            elems: vec!(),
            text: "".to_string(),
        }
    }

    pub fn push(&mut self, s: Box<dyn ElemOfCommand>){
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
            if let Some(d) = delimiter(text)          {ans.push(Box::new(d));}
            else if let Some(e) = end_of_command(text){ans.push(Box::new(e));}
            else{break;};
        };
    
        BlankPart::return_if_valid(ans)
    }
}
