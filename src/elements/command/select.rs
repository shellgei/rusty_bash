//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::word::Word;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, Script, ShellCore, utils};

#[derive(Debug, Clone, Default)]
pub struct SelectCommand {
    text: String,
    name: String,
    has_in: bool,
    values: Vec<Word>,
    do_script: Option<Script>,
    redirects: Vec<Redirect>,
    force_fork: bool,
    lineno: usize,
}

impl Command for SelectCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) -> Result<(), ExecError> {
        if !utils::is_name(&self.name, core) {
            core.db.exit_status = 1;
            ExecError::VariableInvalid(self.name.to_string()).print(core);
            return Ok(());
        }

        let values = match self.has_in {
            true => match self.eval_values(core) {
                Some(vs) => vs,
                None => return Ok(()),
            },
            false => core.db.get_position_params(),
        };

        if values.is_empty() {
            return Ok(());
        }

        self.print(&values, true, core);

        let mut input_value = String::new();
        while let Ok(len) = std::io::stdin().read_line(&mut input_value) {
            if len == 0 {
                break;
            }

            if input_value == "\n" {
                self.print(&values, true, core);
                input_value.clear();
                println!();
                continue;
            }

            input_value = input_value.trim().to_string();
            let num = input_value.parse::<usize>().unwrap_or(values.len() + 1);
            input_value.clear();
            if num == 0 || num > values.len() {
                println!();
                self.print(&values, false, core);
                continue;
            }

            core.db.set_param(&self.name, &values[num - 1], None)?;
            if let Some(mut s) = self.do_script.clone() {
                let _ = s.exec(core);
            }

            self.print(&values, false, core);
        }

        Ok(())
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> {
        &mut self.redirects
    }
    fn get_lineno(&mut self) -> usize {
        self.lineno
    }
    fn set_force_fork(&mut self) {
        self.force_fork = true;
    }
    fn boxed_clone(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
    fn force_fork(&self) -> bool {
        self.force_fork
    }
}

impl SelectCommand {
    fn eval_values(&mut self, core: &mut ShellCore) -> Option<Vec<String>> {
        let mut ans = vec![];
        for w in &mut self.values {
            match w.eval(core) {
                Ok(mut ws) => ans.append(&mut ws),
                Err(e) => {
                    e.print(core);
                    return None;
                }
            }
        }

        Some(ans)
    }

    fn print(&mut self, values: &[String], all: bool, _: &ShellCore) {
        if all {
            for (i, v) in values.iter().enumerate() {
                eprintln!("{}) {}", i + 1, &v);
            }
        }
        eprint!("#? ");
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        if let Ok(Some(w)) = Word::parse(feeder, core, None) {
            ans.name = w.text.clone();
            ans.text += &w.text;
        } else {
            return false;
        }

        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        true
    }

    fn eat_in_part(
        feeder: &mut Feeder,
        ans: &mut Self,
        core: &mut ShellCore,
    ) -> Result<(), ParseError> {
        if !feeder.starts_with("in") {
            return Ok(());
        }

        ans.text += &feeder.consume(2);
        ans.has_in = true;

        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            match Word::parse(feeder, core, None)? {
                Some(w) => {
                    ans.text += &w.text.clone();
                    ans.values.push(w);
                }
                _ => return Ok(()),
            }
        }
    }

    fn eat_end(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        if feeder.starts_with(";") || feeder.starts_with("\n") {
            ans.text += &feeder.consume(1);
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            true
        } else {
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with("select") {
            return Ok(None);
        }
        let mut ans = Self {
            lineno: feeder.lineno,
            text: feeder.consume(6),
            ..Default::default()
        };

        if Self::eat_name(feeder, &mut ans, core) {
            Self::eat_in_part(feeder, &mut ans, core)?;
        } else {
            return Ok(None);
        }

        let _ = Self::eat_end(feeder, &mut ans, core);

        command::eat_blank_lines(feeder, core, &mut ans.text)?;

        if command::eat_inner_script(feeder, core, "do", vec!["done"], &mut ans.do_script, false)? {
            ans.text.push_str("do");
            if let Some(ref mut s) = ans.do_script {
                ans.text.push_str(&s.get_text());
            }
            ans.text.push_str(&feeder.consume(4)); //done

            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
            Ok(Some(ans))
        } else {
            Ok(None)
        }
    }
}
