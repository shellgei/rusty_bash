//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::job::Job;
use crate::operators::ControlOperator;

enum EndStatus{
    UnexpectedSymbol(String),
    NeedMoreLine,
    NormalEnd,
}

#[derive(Debug)]
pub struct Script {
    pub jobs: Vec<Job>,
    pub job_ends: Vec<ControlOperator>,
    pub text: String,
}

impl Script {
    pub fn exec(&mut self, core: &mut ShellCore) {
        let mut counter = 0;
        for j in self.jobs.iter_mut() {
            if self.job_ends[counter] == ControlOperator::BgAnd {
            //    j.is_bg = true;
                j.text += " &";
                j.exec_bg(core);
            }else{
                j.exec(core);
            }
            counter += 1;

            if core.return_flag {
                core.return_flag = false;
                return;
            }
        }
    }

    pub fn new() -> Script{
        Script {
            jobs: vec![],
            job_ends: vec![],
            text: "".to_string(),
        }
    }

    fn eat_job(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Script) -> bool { 
        if let Some(j) =  Job::parse(feeder, core) {
            ans.text += &j.text.clone();
            ans.jobs.push(j);
            true
        }else{
            false
        }
    }

    fn check_nest(feeder: &mut Feeder, ends: &Vec<&str>, other_ends: &Vec<&str>, empty: bool) -> EndStatus {
        if let Some(end) = ends.iter().find(|e| feeder.starts_with(e)) {
            if end == &";;" || end == &";&" || end == &";;&" {
                return EndStatus::NormalEnd;
            }
            if empty {
                return EndStatus::UnexpectedSymbol(end.to_string());
            }
            return EndStatus::NormalEnd;
        }

        if let Some(end) = other_ends.iter().find(|e| feeder.starts_with(e)) {
            return EndStatus::UnexpectedSymbol(end.to_string());
        }
        return EndStatus::NeedMoreLine;
    }

    fn check_end(feeder: &mut Feeder, core: &mut ShellCore, empty: bool) -> EndStatus {
        let ends = vec![")", "}", "then", "else", "fi", "elif", "do", "done"];

        if let Some(begin) = core.nest.pop() {
            core.nest.push(begin.clone());
            return match begin.as_ref() {
                "(" => Self::check_nest(feeder, &vec![")"], &ends, empty),
                "{" => Self::check_nest(feeder, &vec!["}"], &ends, empty),
                "if" | "elif" => Self::check_nest(feeder, &vec!["then"], &ends, empty),
                "then" => Self::check_nest(feeder, &vec!["else", "fi", "elif"], &ends, empty),
                "else" => Self::check_nest(feeder, &vec!["fi"], &ends, empty),
                "while" => Self::check_nest(feeder, &vec!["do"], &ends, empty),
                "do" => Self::check_nest(feeder, &vec!["done"], &ends, empty),
                "_)" => Self::check_nest(feeder, &vec![";;", ";&", ";;&"], &ends, empty), // pattern in case
                _ => EndStatus::NormalEnd,
            };
        }

        if let Some(token) = ends.iter().find(|e| feeder.starts_with(e)) {
            return EndStatus::UnexpectedSymbol(token.to_string());
        }

        return EndStatus::NormalEnd;
    }

    fn eat_job_end(feeder: &mut Feeder, ans: &mut Script) {
        let (n, op) = feeder.scanner_control_op();
        if let Some(p) = op {
            ans.job_ends.push(p.clone());
            if &p == &ControlOperator::Semicolon || &p == &ControlOperator::BgAnd {
                ans.text += &feeder.consume(n);
            }
        }else{
            ans.job_ends.push(ControlOperator::NoChar);
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        if feeder.len() == 0 {
            return None;
        };
    
        let mut ans = Script::new();
        loop{ 
            while Self::eat_job(feeder, core, &mut ans){
                Self::eat_job_end(feeder, &mut ans);
            }
            ans.text += &feeder.consume_blank_return();

            match Self::check_end(feeder, core, ans.jobs.len() == 0) {
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
                    if let Some(s) = core.nest.last() {
                        if s == "_)" {
                            return Some( ans )
                        }
                    }
                    if ans.jobs.len() == 0 {
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
