//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error_message, Feeder, ShellCore};
use crate::elements::command;
use super::word::Word;

#[derive(Debug, Clone)]
pub enum CondExprType {
    FileCheck(String, Option<Word>),
}

#[derive(Debug, Clone)]
pub struct ConditionalExpr {
    pub text: String,
    pub err_msg: String, 
    pub expr: Option<CondExprType>,
}

impl ConditionalExpr {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<i32> {
        Some(0)
    }

    pub fn new() -> ConditionalExpr {
        ConditionalExpr {
            text: String::new(),
            err_msg: String::new(),
            expr: None,
        }
    }

    fn eat_file_check(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_file_check_option(core);
        if len != 2 {
            return false;
        }
        let option = feeder.consume(2);
        ans.text += &option.clone();
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        if feeder.starts_with("]]") {
            ans.err_msg = "unexpected argument".to_string();
            return true;
        }

        match Word::parse(feeder, core, false) {
            Some(w) => {
                ans.text += &w.text.clone();
                ans.expr = Some(CondExprType::FileCheck(option, Some(w)));
            },
            None => {
                ans.err_msg = "unexpected argument".to_string();
            },
        }
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<ConditionalExpr> {
        let mut ans = ConditionalExpr::new();

        if Self::eat_file_check(feeder, &mut ans, core) {
            Some(ans)
        }else{
            None
        }
    }
}
