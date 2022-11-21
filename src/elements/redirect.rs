//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::ShellCore;
use crate::scanner::*;
use crate::element_list::RedirectOp;
use crate::elements::arg::Arg;
use crate::abst_elems::CommandElem;

pub struct Redirect {
    pub text: String,
    pub pos: DebugInfo,
    pub left_fd: i32,
    pub right_fd: i32,
    pub redirect_type: RedirectOp,
    pub path: String,
    pub right_arg: Option<Arg>,
}

impl Redirect {
    fn new(text :&Feeder) -> Redirect {
        Redirect {
            text: String::new(),
            pos: DebugInfo::init(text),
            left_fd: -1,
            right_fd: -1,
            redirect_type: RedirectOp::NoRedirect,
            path: String::new(),
            right_arg: None,
        }
    }

    pub fn eval(&mut self, conf: &mut ShellCore) -> String {
        if let Some(a) = &mut self.right_arg {
            let strings = a.eval(conf);
            if strings.len() == 1 {
                return a.eval(conf)[0].clone();
            }/*else if strings.len() > 1 {
                eprintln!("bash: {}: ambiguous redirect", &a.text);
            }*/
        }

        String::new()
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Redirect> {
        let mut ans = Redirect::new(text);
        let backup = text.clone();
        let pos = scanner_number(text, 0);
        if pos > 0 {
            if let Ok(num) = text.from_to(0, pos).parse::<i32>() {
                ans.left_fd = num;
                ans.text += &text.consume(pos);
            }else{
                return None;
            }
        }

        let (pos, red) = scanner_redirect(text);
        if pos == 0 {
            text.rewind(backup);
            return None;
        }

        ans.redirect_type = red.unwrap();
        ans.text += &text.consume(pos);
        ans.text += &text.consume_blank();

        if ans.left_fd == -1 {
            if ans.redirect_type == RedirectOp::Input {
                ans.left_fd = 0;
            }else if ans.redirect_type == RedirectOp::Output {
                ans.left_fd = 1;
            }
        }


        if let Some(a) = Arg::parse(text, conf, false, false) {
            ans.text += &a.text.clone();
            ans.right_arg = Some(a);
        }else{
            text.rewind(backup);
            return None;
        };

        Some(ans)
    }
}
