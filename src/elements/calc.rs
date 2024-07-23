//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command;
use crate::elements::subword::Subword;
use super::word::Word;

enum CalcElement {
    Op(String),
    //Var(Box<dyn Subword>),
    Num(String),
}

#[derive(Debug, Clone)]
pub struct Calc {
    pub text: String,
    pub elements: Vec<String>,
}

impl Calc {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<String> {
        None
    }

    pub fn new() -> Calc {
        Calc {
            text: String::new(),
            elements: vec![],
        }
    }

    fn eat_sign_or_interger(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let mut text = String::new();
        if feeder.starts_with("+") || feeder.starts_with("-") {
            text = feeder.consume(1);
        }

        let mut nums_len = feeder.scanner_nonnegative_integer(core);
        if nums_len > 0 {
            text += &feeder.consume(nums_len);
        }

        /*
        if let Some(a) = BracedParam::parse(feeder, core){
            ans.text += a.get_text();
            ans.subwords.push(Box::new(a));
            true
        }else{
            false
        }*/

        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Calc> {
        let mut ans = Calc::new();

        loop {
            Self::eat_sign_or_interger(feeder, &mut ans, core);
        }

        None
    }
}
