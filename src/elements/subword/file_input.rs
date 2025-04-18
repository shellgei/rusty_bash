//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command;
use crate::elements::io::redirect::Redirect;
use crate::elements::subword::Subword;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Default)]
pub struct FileInput {
    pub text: String,
    redirect: Redirect,
}

impl Subword for FileInput {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let args = self.redirect.right.eval(core)?;
        if args.len() != 1 {
            return Err(ExecError::AmbiguousRedirect(self.redirect.right.text.clone()));
        }

        let file = match File::open(&args[0]) {
            Ok(f) => f,
            Err(e) => return Err(ExecError::Other(e.to_string())),
        };
        let reader = BufReader::new(file);
        self.text.clear();
        for line in reader.lines() {
            match line {
                Ok(ln) => {
                    self.text += &ln;
                    self.text += " ";
                },
                Err(e) => return Err(ExecError::Other(e.to_string())),
            }
        }

        self.text.pop();
        Ok(())
    }
}

impl FileInput {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("$(") {
            return Ok(None);
        }
        feeder.set_backup();
        let mut ans = Self::default();

        ans.text = feeder.consume(2);

        if let Err(e) = command::eat_blank_lines(feeder, core, &mut ans.text) {
            feeder.rewind();
            return Err(e);
        }

        let mut redirects = vec![];
        if let Err(e) = command::eat_redirects(feeder, core, &mut redirects, &mut ans.text) {
            feeder.rewind();
            return Err(e);
        }

        if redirects.len() != 1 
        || redirects[0].symbol != "<" {
            feeder.rewind();
            return Ok(None);
        }

        if let Err(e) = command::eat_blank_lines(feeder, core, &mut ans.text) {
            feeder.rewind();
            return Err(e);
        }

        ans.redirect = redirects.pop().unwrap();

        if ! feeder.starts_with(")") {
            feeder.rewind();
            return Ok(None);
        }

        ans.text += &feeder.consume(1);
        feeder.pop_backup();
        Ok(Some(ans))
    }
}
