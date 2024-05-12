//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use crate::elements::command;
use crate::elements::word::Word;
use nix::unistd::Pid;
use super::{Command, Pipe, Redirect};

#[derive(Debug, Clone)]
pub struct CaseCommand {
    pub text: String,
    pub word: Option<Word>,
    pub patterns_script_end: Vec<(Vec<Word>, Script, String)>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for CaseCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if self.force_fork || pipe.is_connected() {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core);
            None
        }
    }

    fn run(&mut self, core: &mut ShellCore, _: bool) {
        for e in &mut self.patterns_script_end {
            for pattern in &e.0 {
                let t = pattern.clone().text;
                let w = self.word.clone().unwrap().text;
                if t == w {
                    e.1.exec(core);

                    if e.2 == ";;" {
                        return;
                    }
                }
            }
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
}

impl CaseCommand {
    fn new() -> Self {
        CaseCommand {
            text: String::new(),
            word: None,
            patterns_script_end: vec![],
            redirects: vec![],
            force_fork: false,
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        let w = match Word::parse(feeder, core) {
            Some(w) => w,
            _       => return false,
        };

        ans.text += &w.text;
        ans.word = Some(w);
        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        true
    }

    fn eat_patterns(feeder: &mut Feeder, ans: &mut Vec<Word>, text: &mut String, core: &mut ShellCore) -> bool {
        loop {
            match Word::parse(feeder, core) {
                Some(w) => {
                    *text += &w.text;
                    ans.push(w)
                },
                _       => return false,
            };
    
            command::eat_blank_with_comment(feeder, core, text);

            if feeder.starts_with(")") {
                break;
            }

            if feeder.starts_with("|") {
                *text += &feeder.consume(1);
                command::eat_blank_with_comment(feeder, core, text);
            }
        }

        ans.len() != 0
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<CaseCommand> {
        if ! feeder.starts_with("case") {
            return None;
        }

        let mut ans = Self::new();
        ans.text = feeder.consume(4);

        if ! Self::eat_word(feeder, &mut ans, core) 
        || ! feeder.starts_with("in") {
            return None;
        }
        ans.text += &feeder.consume(2);

        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if feeder.starts_with("\n") {
                ans.text += &feeder.consume(1);
            }

            if feeder.starts_with("esac") {
                ans.text += &feeder.consume(4);
                break;
            }

            if feeder.len() == 0 {
                match feeder.feed_additional_line(core) {
                    true  => continue,
                    false => return None,
                }
            }
            let mut patterns = vec![];
            if ! Self::eat_patterns(feeder, &mut patterns, &mut ans.text, core) {
                return None;
            }

            let mut script = None;
            if command::eat_inner_script(feeder, core, ")", vec![";;&", ";;", ";&"], &mut script, false) {
                ans.text.push_str(&script.as_ref().unwrap().get_text());
                let end_len = if feeder.starts_with(";;&") {
                    3
                }else{
                    2
                };
                let end = feeder.consume(end_len);
                ans.text.push_str(&end);
                ans.patterns_script_end.push( (patterns, script.unwrap(), end ) );
            }else{
                return None;
            }
        }

        if ans.patterns_script_end.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
