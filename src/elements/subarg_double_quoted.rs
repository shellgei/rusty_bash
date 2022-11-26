//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;

use crate::abst_elems::ArgElem;
use crate::elements::subarg_string_double_quoted::SubArgStringDoubleQuoted;
use crate::elements::subarg_variable::SubArgVariable;
use crate::elements::subarg_command_substitution::SubArgCommandSubstitution;
use crate::utils::combine;

pub struct SubArgDoubleQuoted {
    pub text: String,
    pub pos: DebugInfo,
    pub subargs: Vec<Box<dyn ArgElem>>,
}

impl ArgElem for SubArgDoubleQuoted {
    fn eval(&mut self, conf: &mut ShellCore, as_value: bool) -> Vec<Vec<String>> {
        conf.in_double_quot = true;

        let mut vvv = vec![];
        for sa in &mut self.subargs {
            vvv.push(sa.eval(conf, as_value));
        };

        let mut strings = vec![];
        for ss in vvv {
            strings = combine(&mut strings, ss);
        }

        let mut ans = vec![];
        for ss in strings {
            let mut anselem = vec![];
            for s in ss {
                let x = s.replace("*", "\\*");
                anselem.push(x);
            }
            ans.push(anselem);
        }

        conf.in_double_quot = false;
        ans
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn permit_lf(&self) -> bool {true}
}


impl SubArgDoubleQuoted {
/* parser for a string such as "aaa${var}" */
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, is_value: bool) -> Option<SubArgDoubleQuoted> {
        if ! text.starts_with("\"") {
            return None;
        };

        let mut ans = SubArgDoubleQuoted {
            text: "".to_string(),
            pos: DebugInfo::init(text),
            subargs: vec![],
        };
    
        ans.text += &text.consume(1);
    
        loop {
            if let Some(a) = SubArgCommandSubstitution::parse(text, conf, is_value) {
                ans.text += &a.text.clone();
                ans.subargs.push(Box::new(a));
            }else if let Some(a) = SubArgVariable::parse(text) {
                ans.text += &a.text.clone();
                ans.subargs.push(Box::new(a));
            }else if let Some(a) = SubArgStringDoubleQuoted::parse(text, conf, is_value) {
                ans.text += &a.text.clone();
                ans.subargs.push(Box::new(a));
            }

            if text.starts_with("\"") {
            //if text.len() > 0 && text.nth_is(0, "\"") {
                ans.text += &text.consume(1);
                break;
            }
        }
    
        Some(ans)
    }
}

