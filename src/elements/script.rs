//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::job::Job;

enum EndStatus{
    UnexpectedSymbol(String),
    NeedMoreLine,
    NormalEnd,
}

#[derive(Debug)]
pub struct Script {
    pub list: Vec<Job>,
    pub text: String,
}

impl Script {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for j in self.list.iter_mut() {
            j.exec(core);

            if core.return_flag {
                core.return_flag = false;
                return;
            }
        }
    }

    pub fn new() -> Script{
        Script {
            list: vec![],
            text: "".to_string(),
        }
    }

    fn eat_job(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Script) -> bool { 
        ans.text += &feeder.consume_blank();
        if let Some(j) =  Job::parse(feeder, core) {
            ans.text += &j.text.clone();
            ans.list.push(j);
            true
        }else{
            false
        }
    }

    fn check_nest(feeder: &mut Feeder, ends: &Vec<&str>, ngs: &Vec<&str>, empty: bool) -> EndStatus {
        for end in ends {
            if feeder.starts_with(end){
                if empty {
                    return EndStatus::UnexpectedSymbol(end.to_string());
                }
                return EndStatus::NormalEnd;
            }
        }

        for ng in ngs {
            if feeder.starts_with(ng){
                return EndStatus::UnexpectedSymbol(ng.to_string());
            }
        }

        return EndStatus::NeedMoreLine;
    }

    fn check_end(feeder: &mut Feeder, core: &mut ShellCore, empty: bool) -> EndStatus {
        if let Some(begin) = core.nest.pop() {
            core.nest.push(begin.clone());
            if begin == "(" {
                return Self::check_nest(feeder, &vec![")"], &vec!["}"], empty);
            }else if begin == "{" {
                return Self::check_nest(feeder, &vec!["}"], &vec![")"], empty);
            }else{
                return EndStatus::NormalEnd;
            }
        }

        for token in vec![")", "}", "then", "elif", "fi", "else", "do", "esac", "done"] {
            if feeder.starts_with(token){
                return EndStatus::UnexpectedSymbol(token.to_string());
            }
        }

        return EndStatus::NormalEnd;
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        if feeder.len() == 0 {
            return None;
        };
    
        let mut ans = Script::new();
        loop{ 
            if Self::eat_job(feeder, core, &mut ans){
                continue;
            }
            ans.text += &feeder.consume_blank_return();

            match Self::check_end(feeder, core, ans.list.len() == 0) {
                EndStatus::UnexpectedSymbol(s) => {
                    eprintln!("Unexpected token: {}", s);
                    core.set_var("?", "2");
                    feeder.consume(feeder.len());
                    return None;
                },
                EndStatus::NeedMoreLine => {
                    if ! feeder.feed_additional_line(core) {
                        feeder.consume(feeder.len());
                        return None;
                    }
                },
                EndStatus::NormalEnd => {
                    if ans.list.len() == 0 {
                        core.set_var("?", "2");
                        feeder.consume(feeder.len());
                        return None;
                    }
                    return Some( ans )
                }
            }
        }
    }
}
