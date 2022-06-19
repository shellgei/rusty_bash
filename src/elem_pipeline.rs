//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_script_elem::ScriptElem;
use crate::Command;
use crate::elem_arg_delimiter::ArgDelimiter;
use nix::unistd::pipe;
use crate::scanner::scanner_end_paren;
use crate::elem_compound_paren::CompoundParen;
use crate::elem_compound_brace::CompoundBrace;

pub struct Pipeline {
    pub commands: Vec<Box<dyn ScriptElem>>,
    pub text: String,
}

impl ScriptElem for Pipeline {

    fn exec(&mut self, conf: &mut ShellCore) {
        let len = self.commands.len();
        let mut prevfd = -1;
        for (i, c) in self.commands.iter_mut().enumerate() {
            let mut p = (-1, -1);
            if i != len-1 {
                p = pipe().expect("Pipe cannot open");
            };
            c.set_pipe(p.0, p.1, prevfd);

            let _ = c.exec(conf);
            c.set_parent_io();
            prevfd = c.get_pipe_end();
        }

        for c in &self.commands {
            if let Some(p) = c.get_pid() {
                self.wait(p, conf);
            }
        }
    }
}

impl Pipeline {
    pub fn new() -> Pipeline{
        Pipeline {
            commands: vec!(),
            text: "".to_string(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Pipeline> {
        let mut ans = Pipeline::new();

        loop {
            let eocs;
            if let Some(mut c) = Command::parse(text, conf) {
                eocs = c.get_eoc_string();
                ans.text += &c.text.clone();
                ans.commands.push(Box::new(c));
            }else if let Some(mut c) = CompoundParen::parse(text, conf) {
                eocs = c.get_eoc_string();
                ans.text += &c.text.clone();
                ans.commands.push(Box::new(c));
            }else if let Some(mut c) = CompoundBrace::parse(text, conf) {
                eocs = c.get_eoc_string();
                ans.text += &c.text.clone();
                ans.commands.push(Box::new(c));
            }else{
                break;
            }

            if eocs != "|" {
                break;
            }

            if let Some(d) = ArgDelimiter::parse(text) {
                ans.text += &d.text.clone();
            }

            if eocs == "|" && text.len() == 1 && text.nth(0) == '\n' {
                text.consume(1);
                text.feed_additional_line(conf);
            }

            if scanner_end_paren(text, 0) == 1 {
                break;
            }
        }

        if ans.commands.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
