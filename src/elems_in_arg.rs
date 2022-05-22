//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::CommandPart;
use crate::ShellCore;
use crate::elems_in_command::Arg;

pub trait ArgElem {
    fn eval(&self, _conf: &mut ShellCore) -> Vec<String> {
        vec!()
    }

    fn text(&self) -> String;
}

pub struct VarName {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for VarName {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn eval(&self, _conf: &mut ShellCore) -> Vec<String> {
        vec!(self.text.clone())
    }
}

pub struct SubArg {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArg {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn eval(&self, _conf: &mut ShellCore) -> Vec<String> {
        vec!(self.text.clone())
    }
}

pub struct SubArgDoubleQuoted {
    pub text: String,
    pub pos: DebugInfo,
    pub subargs: Vec<Box<dyn ArgElem>>
}

impl ArgElem for SubArgDoubleQuoted {
    fn eval(&self, conf: &mut ShellCore) -> Vec<String> {
        let mut text = "".to_string();
        for a in &self.subargs {
            let sub = a.eval(conf);
            text += &sub[0];
        };

        let s = text.replace("\\", "\\\\").replace("*", "\\*"); 
        vec!(s)
    }

    fn text(&self) -> String {
        self.text.clone()
    }

    /*
    fn get_length(&self) -> usize {
        self.text.len()
    }
    */
}

pub struct SubArgSingleQuoted {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgSingleQuoted {
    fn eval(&self, _conf: &mut ShellCore) -> Vec<String> {
        let strip = self.text[1..self.text.len()-1].to_string();
        let s = strip.replace("\\", "\\\\").replace("*", "\\*"); 
        vec!(s)
    }

    fn text(&self) -> String {
        self.text.clone()
    }

    /*
    fn get_length(&self) -> usize {
        self.text.len()
    }
    */
}

pub struct SubArgBraced {
    pub text: String,
    pub pos: DebugInfo,
    pub args: Vec<Arg>
}

impl ArgElem for SubArgBraced {
    fn eval(&self, conf: &mut ShellCore) -> Vec<String> {
        if self.args.len() == 0{
            return vec!("{}".to_string());
        }else if self.args.len() == 1{
            return vec!("{".to_owned() + &self.args[0].text.clone() + "}");
        };

        let mut ans = vec!();
        for arg in &self.args {
            ans.append(&mut arg.eval(conf));
        };
        ans
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

pub struct SubArgVariable {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgVariable {
    fn eval(&self, conf: &mut ShellCore) -> Vec<String> {
        let name = if self.text.rfind('}') == Some(self.text.len()-1) {
            self.text[2..self.text.len()-1].to_string()
        }else{
            self.text[1..].to_string()
        };
        vec!(conf.get_var(&name))
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

#[derive(Debug)]
pub struct DelimiterInArg {
    pub text: String,
    pub debug: DebugInfo,
}

impl ArgElem for DelimiterInArg {
    fn text(&self) -> String {
        self.text.clone()
    }
}
