//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};

#[derive(Debug, Clone)]
enum CalcElement {
    //Op(String),
    //Var(Box<dyn Subword>),
    Num(String),
}

#[derive(Debug, Clone)]
pub struct Calc {
    pub text: String,
    elements: Vec<CalcElement>,
}

impl Calc {
    pub fn eval(&mut self, _: &mut ShellCore) -> Option<String> {
        for e in &self.elements {
            match e {
                CalcElement::Num(s) => return Some(s.to_string()),
                _ => return None,
            }
        }

        None
    }

    pub fn new() -> Calc {
        Calc {
            text: String::new(),
            elements: vec![],
        }
    }

    fn eat_blank(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        let len = feeder.scanner_blank(core);
        ans.text += &feeder.consume(len);
    }

    fn eat_interger(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_integer(core);
        if len == 0 {
            return false;
        }

        let s = feeder.consume(len);
        ans.text += &s.clone();
        ans.elements.push( CalcElement::Num(s) );

        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Calc> {
        let mut ans = Calc::new();

        loop {
            Self::eat_blank(feeder, &mut ans, core);
            if ! Self::eat_interger(feeder, &mut ans, core) {
                break;
            }
        }

        Self::eat_blank(feeder, &mut ans, core);
        match feeder.starts_with("))") {
            true  => Some(ans),
            false => None,
        }
    }
}
