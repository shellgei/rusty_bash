//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::command;
use super::command::Command;
use super::Pipe;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};
use nix::sys::resource;
use nix::time;
use nix::time::ClockId;
use nix::unistd::Pid;

#[derive(Debug, Clone, Default)]
pub struct Pipeline {
    pub commands: Vec<Box<dyn Command>>,
    pub pipes: Vec<Pipe>,
    pub text: String,
    exclamation: bool,
    pub time: bool,
}

impl Pipeline {
    pub fn exec(
        &mut self,
        core: &mut ShellCore,
        pgid: Pid,
    ) -> (Vec<Option<Pid>>, bool, bool, Option<ExecError>) {
        if self.commands.is_empty() {
            // the case of only '!'
            self.set_time(core);
            return (vec![], self.exclamation, self.time, None);
        }

        let mut prev = -1;
        let mut prevs = vec![];
        let mut pids = vec![];
        let mut pgid = pgid;

        self.set_time(core);

        for (i, p) in self.pipes.iter_mut().enumerate() {
            p.set(prev, pgid);
            p.aux_prevs = prevs;

            match self.commands[i].exec(core, p) {
                Ok(pid) => pids.push(pid),
                Err(e) => return (pids, self.exclamation, self.time, Some(e)),
            }

            if i == 0 && pgid.as_raw() == 0 {
                // 最初のexecが終わったら、pgidにコマンドのPIDを記録
                pgid = pids[0].unwrap();
            }
            prev = p.recv;
            prevs = self.commands[i].get_prev_fds();
        }

        let lastpipe = (!core.db.flags.contains('m')) && core.shopts.query("lastpipe");
        let mut lastp = Pipe::end(prev, pgid, lastpipe);
        lastp.aux_prevs = prevs;
        let result = self.commands[self.pipes.len()].exec(core, &mut lastp);
        if lastpipe {
            lastp.restore_lastpipe();
        }

        match result {
            Ok(pid) => pids.push(pid),
            Err(e) => return (pids, self.exclamation, self.time, Some(e)),
        }

        (pids, self.exclamation, self.time, None)
    }

    fn set_time(&mut self, core: &mut ShellCore) {
        if !self.time {
            return;
        }

        let self_usage = resource::getrusage(resource::UsageWho::RUSAGE_SELF).unwrap();
        let children_usage = resource::getrusage(resource::UsageWho::RUSAGE_CHILDREN).unwrap();

        core.measured_time.user = self_usage.user_time() + children_usage.user_time();
        core.measured_time.sys = self_usage.system_time() + children_usage.system_time();
        core.measured_time.real = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();
    }

    pub fn read_heredoc(
        &mut self,
        feeder: &mut Feeder,
        core: &mut ShellCore,
    ) -> Result<(), ParseError> {
        for command in self.commands.iter_mut() {
            command.read_heredoc(feeder, core)?;
        }
        Ok(())
    }

    pub fn get_one_line_text(&self) -> String {
        let mut ans = String::new();

        if self.exclamation {
            ans += "! ";
        }

        for (i, c) in self.commands.iter().enumerate() {
            ans += &c.get_one_line_text();
            if i < self.pipes.len() {
                ans += &self.pipes[i].text;
            }
        }
        ans
    }

    fn eat_exclamation(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match feeder.starts_with("!") {
            true => ans.text += &feeder.consume(1),
            false => return false,
        }

        ans.exclamation = !ans.exclamation;
        let blank_len = feeder.scanner_blank(core);
        ans.text += &feeder.consume(blank_len);
        true
    }

    fn eat_time(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match feeder.starts_with("time ") || feeder.starts_with("time\t") {
            true => ans.text += &feeder.consume(4),
            false => return false,
        }

        ans.time = true;
        let blank_len = feeder.scanner_blank(core);
        ans.text += &feeder.consume(blank_len);
        true
    }

    fn eat_command(
        feeder: &mut Feeder,
        ans: &mut Pipeline,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        if let Some(command) = command::parse(feeder, core)? {
            ans.text += &command.get_text();
            ans.commands.push(command);

            let blank_len = feeder.scanner_blank(core);
            ans.text += &feeder.consume(blank_len);
            return Ok(true);
        }
        Ok(false)
    }

    fn eat_pipe(feeder: &mut Feeder, ans: &mut Pipeline, core: &mut ShellCore) -> bool {
        if let Some(p) = Pipe::parse(feeder, core) {
            ans.text += &p.text.clone();
            ans.pipes.push(p);
            true
        } else {
            false
        }
    }

    fn eat_blank_and_comment(feeder: &mut Feeder, ans: &mut Pipeline, core: &mut ShellCore) {
        loop {
            let blank_len = feeder.scanner_multiline_blank(core);
            ans.text += &feeder.consume(blank_len); //空白、空行を削除
            let comment_len = feeder.scanner_comment();
            ans.text += &feeder.consume(comment_len); //コメントを削除
            if blank_len + comment_len == 0 {
                //空白、空行、コメントがなければ出る
                break;
            }
        }
    }

    pub fn parse(
        feeder: &mut Feeder,
        core: &mut ShellCore,
    ) -> Result<Option<Pipeline>, ParseError> {
        let mut ans = Pipeline::default();

        while Self::eat_exclamation(feeder, &mut ans, core)
            || Self::eat_time(feeder, &mut ans, core)
        {}

        if !Self::eat_command(feeder, &mut ans, core)? {
            match ans.exclamation || ans.time {
                true => return Ok(Some(ans)),
                false => return Ok(None),
            }
        }

        while Self::eat_pipe(feeder, &mut ans, core) {
            loop {
                Self::eat_blank_and_comment(feeder, &mut ans, core);
                if Self::eat_command(feeder, &mut ans, core)? {
                    break;
                }
                if !feeder.is_empty() {
                    return Ok(None);
                }
                feeder.feed_additional_line(core)?;
            }
        }

        Ok(Some(ans))
    }
}
