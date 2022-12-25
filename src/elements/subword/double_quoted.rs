//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;

use crate::elements::subword::Subword;
use crate::elements::subword::string_double_quoted::SubwordStringDoubleQuoted;
use crate::elements::subword::variable::SubwordVariable;
use crate::elements::subword::command_substitution::SubwordCommandSubstitution;
use crate::utils::combine;

pub struct SubwordDoubleQuoted {
    pub text: String,
    pub pos: DebugInfo,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl Subword for SubwordDoubleQuoted {
    fn eval(&mut self, conf: &mut ShellCore, _: bool) -> Vec<Vec<String>> {
        conf.in_double_quot = true;

        let mut vvv = vec![];
        for sa in &mut self.subwords {
            vvv.push(sa.eval(conf, false)); //not expand in this double quote
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


impl SubwordDoubleQuoted {
/* parser for a string such as "aaa${var}" */
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<SubwordDoubleQuoted> {
        if ! text.starts_with("\"") {
            return None;
        };

        let mut ans = SubwordDoubleQuoted {
            text: "".to_string(),
            pos: DebugInfo::init(text),
            subwords: vec![],
        };
    
        ans.text += &text.consume(1);
    
        loop {
            if let Some(a) = SubwordCommandSubstitution::parse(text, conf) {
                ans.text += &a.text.clone();
                ans.subwords.push(Box::new(a));
            }else if let Some(a) = SubwordVariable::parse(text) {
                ans.text += &a.text.clone();
                ans.subwords.push(Box::new(a));
            }else if let Some(a) = SubwordStringDoubleQuoted::parse(text, conf) {
                ans.text += &a.text.clone();
                ans.subwords.push(Box::new(a));
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

