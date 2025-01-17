//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use crate::error::parse::ParseError;
use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::word::Word;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::error;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug, Clone, Default)]
pub struct ForCommand {
    text: String,
    name: String,
    has_in: bool,
    has_arithmetic: bool,
    values: Vec<Word>,
    arithmetics: Vec<Option<ArithmeticExpr>>,
    do_script: Option<Script>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for ForCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        core.loop_level += 1;

        let ok = match self.has_arithmetic {
            true  => self.run_with_arithmetic(core),
            false => self.run_with_values(core),
        };

        if ! ok && core.db.exit_status == 0 {
            core.db.exit_status = 1;
        }

        core.loop_level -= 1;
        if core.loop_level == 0 {
            core.break_counter = 0;
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl ForCommand {
    fn eval_values(&mut self, core: &mut ShellCore) -> Option<Vec<String>> {
        let mut ans = vec![];
        for w in &mut self.values {
            match w.eval(core) {
                Ok(mut ws) => ans.append(&mut ws),
                Err(e)     => {
                    error::print(&e, core);
                    return None;
                },
            }
        }

        Some(ans)
    }

    fn run_with_values(&mut self, core: &mut ShellCore) -> bool {
        let values = match self.has_in {
            true  => match self.eval_values(core) {
                Some(vs) => vs,
                None     => return false,
            },
            false => core.db.get_position_params(),
        };

        for p in values {
            if core.sigint.load(Relaxed) {
                return false;
            }

            if let Err(e) = core.db.set_param(&self.name, &p, None) {
                core.db.exit_status = 1;
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
            }

            if core.continue_counter > 0 {
                core.continue_counter -= 1;
                continue;
            }

            self.do_script.as_mut().unwrap().exec(core);

            if core.break_counter > 0 {
                core.break_counter -= 1;
                break;
            }
        }
        true
    }

    fn eval_arithmetic(a: &mut Option<ArithmeticExpr>, core: &mut ShellCore) -> (bool, String) {
        if a.is_none() {
            return (true, "1".to_string());
        }

        match a.clone().unwrap().eval(core) {
            Ok(n) => return (true, n),
            _     => return (false, "0".to_string()), 
        }
    }

    fn run_with_arithmetic(&mut self, core: &mut ShellCore) -> bool {
        let (ok, _) = Self::eval_arithmetic(&mut self.arithmetics[0], core);
        if ! ok {
            return false;
        }

        loop {
            if core.sigint.load(Relaxed) {
                return false;
            }

            let (ok, val) = Self::eval_arithmetic(&mut self.arithmetics[1], core);
            if val == "0" {
                return ok;
            }

            self.do_script.as_mut().unwrap().exec(core);

            if core.break_counter > 0 {
                core.break_counter -= 1;
                break;
            }

            let (ok, _) = Self::eval_arithmetic(&mut self.arithmetics[2], core);
            if ! ok {
                return false;
            }
        }
        true
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        let len = feeder.scanner_name(core);
        if len == 0 {
            return false;
        }

        ans.name = feeder.consume(len);
        ans.text += &ans.name.clone();
        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        true
    }

    fn eat_arithmetic(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if ! feeder.starts_with("((") {
            return false;
        }
        ans.text += &feeder.consume(2);
        ans.has_arithmetic = true;
 
        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if feeder.len() == 0 {
                match feeder.feed_additional_line(core).is_ok() {
                    true  => continue,
                    false => return false,
                }
            }

            let a = ArithmeticExpr::parse(feeder, core, true);
            if a.is_some() {
                ans.text += &a.as_ref().unwrap().text.clone();
            }
            ans.arithmetics.push(a);

            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if feeder.starts_with(";") {
                if ans.arithmetics.len() >= 3 {
                    return false;
                }
                ans.text += &feeder.consume(1);
            }else if feeder.starts_with("))") {
                if ans.arithmetics.len() != 3 {
                    return false;
                }
                ans.text += &feeder.consume(2);
                return ans.arithmetics.len() == 3;
            }else {
                return false;
            }
        }
    }

    fn eat_in_part(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Result<(), ParseError> {
        if ! feeder.starts_with("in") {
            return Ok(());
        }

        ans.text += &feeder.consume(2);
        ans.has_in = true;

        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            match Word::parse(feeder, core, false)? {
                Some(w) => {
                    ans.text += &w.text.clone();
                    ans.values.push(w);
                },
                _    => return Ok(()),
            }
        }
    }

    fn eat_end(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        if feeder.starts_with(";") || feeder.starts_with("\n") {
            ans.text += &feeder.consume(1);
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
        -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("for") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text = feeder.consume(3);

        if Self::eat_name(feeder, &mut ans, core) {
            Self::eat_in_part(feeder, &mut ans, core)?;
        }else if ! Self::eat_arithmetic(feeder, &mut ans, core) {
            return Ok(None);
        }

        if ! Self::eat_end(feeder, &mut ans, core) {
            return Ok(None);
        }

        if feeder.len() == 0 {
            feeder.feed_additional_line(core)?;
        }

        while command::eat_blank_with_comment(feeder, core, &mut ans.text) {}

        if command::eat_inner_script(feeder, core, "do", vec!["done"],  &mut ans.do_script, false)? {
            ans.text.push_str("do");
            ans.text.push_str(&ans.do_script.as_mut().unwrap().get_text());
            ans.text.push_str(&feeder.consume(4)); //done

            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
            Ok(Some(ans))
        }else{
            Ok(None)
        }
    }
}
