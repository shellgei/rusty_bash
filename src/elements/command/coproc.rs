//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{Command, Pipe, Redirect};
use crate::elements::command;
use crate::elements::command::{BraceCommand, IfCommand, ParenCommand, WhileCommand};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils;
use crate::{Feeder, ShellCore};
use nix::unistd::Pid;

#[derive(Debug, Clone, Default)]
pub struct Coprocess {
    pub text: String,
    name: String,
    command: Option<Box<dyn Command>>,
    force_fork: bool,
    _dummy: Vec<Redirect>,
    lineno: usize,
}

impl Command for Coprocess {
    fn exec(&mut self, core: &mut ShellCore, _: &mut Pipe) -> Result<Option<Pid>, ExecError> {
        dbg!("{:?}", &self);
        /*
        if core.break_counter > 0 || core.continue_counter > 0 {
            return Ok(None);
        }

        core.db
            .functions
            .insert(self.name.to_string(), self.clone());
        */
        Ok(None)
    }

    fn run(&mut self, _: &mut ShellCore, _: bool) -> Result<(), ExecError> {
        Ok(())
    }
    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> {
        &mut self._dummy
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

    fn pretty_print(&mut self, indent_num: usize) {
        self.pretty_print(indent_num);
    }
}

impl Coprocess {
    pub fn pretty_print(&mut self, indent_num: usize) {
        println!("{} () ", self.name);
        for com in self.command.iter_mut() {
            com.pretty_print(indent_num);
        }
    }

    /*
    pub fn run_as_command(&mut self, args: &mut [String], core: &mut ShellCore) {
        if ! core.db.exist("FUNCNAME") {
            if  core.script_name == "-" {
                let _ = core.db.init_array("FUNCNAME", None, Some(0), false);
            }else {
                let _ = core.db.init_array("FUNCNAME", Some(vec!["main".to_string()]),
                                           Some(0), false);
            }
        }

        let mut array = core.db.get_vec("FUNCNAME", false).unwrap(); //TODO: implement array push
        array.insert(0, args[0].clone()); //TODO: We must put the name not only in 0 but also 1..
        let _ = core.db.init_array("FUNCNAME", Some(array.clone()), None, false);

        let mut linenos = core.db.get_vec("BASH_LINENO", false).unwrap();
        let lineno = core.db.get_param("LINENO").unwrap_or("0".to_string());
        linenos.insert(0, lineno.to_string());
        let _ = core.db.init_array("BASH_LINENO", Some(linenos.clone()), None, false);

        let mut source = core.db.get_vec("BASH_SOURCE", false).unwrap();
        source.insert(0, self.file.clone());
        let _ = core.db.init_array("BASH_SOURCE", Some(source.clone()), None, false);

        args[0] = core.db.position_parameters[0][0].clone();
        core.db.position_parameters.push(args.to_vec());

        let mut dummy = Pipe::new("|".to_string());

        core.source_function_level += 1;
        if let Err(e) = self.command.as_mut().unwrap().exec(core, &mut dummy) {
            e.print(core);
        }
        core.return_flag = false;
        core.source_function_level -= 1;

        core.db.position_parameters.pop();

        array.remove(0);
        if array.is_empty() 
        || ( core.script_name != "-" && array[0] == "main" ) {
            let _ = core.db.unset("FUNCNAME", Some(0));
        }else {
            let _ = core.db.init_array("FUNCNAME", Some(array), Some(0), false);
        }

        linenos.remove(0);
        source.remove(0);
        let _ = core.db.init_array("BASH_LINENO", Some(linenos), None, false);
        let _ = core.db.init_array("BASH_SOURCE", Some(source), None, false);
    }
    */

    fn eat_header(&mut self, feeder: &mut Feeder, core: &mut ShellCore) -> Result<(), ParseError> {
        self.text += &feeder.consume(6);
        command::eat_blank_with_comment(feeder, core, &mut self.text);

        let len = feeder.scanner_name(core);
        self.name = feeder.consume(len).to_string();
        self.text += &self.name;

        if self.name.is_empty() && utils::reserved(&self.name) {
            return Err(ParseError::UnexpectedSymbol("coproc".to_string()));
        }

        command::eat_blank_with_comment(feeder, core, &mut self.text);
        Ok(())
    }

    fn eat_body(&mut self, feeder: &mut Feeder, core: &mut ShellCore) -> Result<(), ParseError> {
        self.command = if let Some(a) = IfCommand::parse(feeder, core)? {
            Some(Box::new(a))
        } else if let Some(a) = ParenCommand::parse(feeder, core, false)? {
            Some(Box::new(a))
        } else if let Some(a) = BraceCommand::parse(feeder, core)? {
            Some(Box::new(a))
        } else if let Some(a) = WhileCommand::parse(feeder, core)? {
            Some(Box::new(a))
        } else {
            None
        };

        if let Some(c) = &self.command {
            self.text += &c.get_text();
        }
        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("coproc") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.lineno = feeder.lineno;

        ans.eat_header(feeder, core)?;
        ans.eat_body(feeder, core)?;

        if ans.command.is_some() {
            Ok(Some(ans))
        } else {
            Ok(None)
        }
    }
}
