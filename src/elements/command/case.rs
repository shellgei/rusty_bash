//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use crate::error::parse::ParseError;
use crate::elements::command;
use crate::elements::word::Word;
use crate::utils::glob;
use super::{Command, Redirect};

#[derive(Debug, Clone)]
pub struct CaseCommand {
    pub text: String,
    pub word: Option<Word>,
    pub patterns_script_end: Vec<(Vec<Word>, Script, String)>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for CaseCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        let mut next = false;
        let word = self.word.clone().unwrap();

        if core.db.flags.contains('x') {
            let ps4 = core.get_ps4();
            eprintln!("\r{} case {} in\r", ps4, word.text);
        }

        let w = match word.eval_for_case_word(core) {
            Some(w) => w, 
            _       => "".to_string(),
        };

        let extglob = core.shopts.query("extglob");

        for e in &mut self.patterns_script_end {
            for pattern in &mut e.0 {
                let p = match pattern.eval_for_case_pattern(core) {
                    Some(p) => p, 
                    _       => continue,
                };

                if glob::parse_and_compare(&w, &p, extglob) || next {
                    e.1.exec(core);

                    if e.2 == ";;" {
                        return;
                    }
                    next = e.2 == ";&";
                }
            }
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
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

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        let w = match Word::parse(feeder, core, false)? {
            Some(w) => w,
            _       => return Ok(false),
        };

        ans.text += &w.text;
        ans.word = Some(w);
        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        Ok(true)
    }

    fn eat_patterns(feeder: &mut Feeder, ans: &mut Vec<Word>, text: &mut String, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        if feeder.starts_with("(") {
            *text += &feeder.consume(1);
        }

        command::eat_blank_with_comment(feeder, core, text);
        loop {
            if let Some(w) = Word::parse(feeder, core, false)? {
                *text += &w.text;
                ans.push(w)
            }else{
                return Ok(false);
            }
    
            command::eat_blank_with_comment(feeder, core, text);

            if feeder.starts_with(")") {
                break;
            }

            if feeder.starts_with("|") {
                *text += &feeder.consume(1);
                command::eat_blank_with_comment(feeder, core, text);
            }
        }

        Ok(ans.len() != 0)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
        -> Result<Option<CaseCommand>, ParseError> {
        if ! feeder.starts_with("case") {
            return Ok(None);
        }

        let mut ans = Self::new();
        ans.text = feeder.consume(4);

        if ! Self::eat_word(feeder, &mut ans, core)?
        || ! feeder.starts_with("in") {
            return Ok(None);
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
                feeder.feed_additional_line(core)?;
            }
            let mut patterns = vec![];
            if ! Self::eat_patterns(feeder, &mut patterns, &mut ans.text, core)? {
                break;
            }

            let mut script = None;
            if command::eat_inner_script(feeder, core, ")", vec![";;&", ";;", ";&"], &mut script, true)? {
                ans.text.push_str(")");
                ans.text.push_str(&script.as_ref().unwrap().get_text());
                let end_len = if feeder.starts_with(";;&") { 3 }else{ 2 };
                let end = feeder.consume(end_len);
                ans.text.push_str(&end);
                ans.patterns_script_end.push( (patterns, script.unwrap(), end ) );
            }else{
                return Ok(None);
            }
        }

        if ans.patterns_script_end.len() > 0 {
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
            Ok(Some(ans))
        }else{
            Ok(None)
        }
    }
}
