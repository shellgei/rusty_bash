//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use super::command;
use super::command::Command;
use super::Pipe;
use nix::unistd::Pid;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub commands: Vec<Box<dyn Command>>,
    pub pipes: Vec<Pipe>,
    pub text: String,
}

impl Pipeline {
    pub fn exec(&mut self, core: &mut ShellCore, pgid: Pid) -> Vec<Option<Pid>> {
        if core.sigint.load(Relaxed) { //以下4行追加
            core.set_param("?", "130");
            return vec![];
        }

        let mut prev = -1;
        let mut pids = vec![];
        let mut pgid = pgid;
        for (i, p) in self.pipes.iter_mut().enumerate() {
            p.set(prev, pgid);
            pids.push(self.commands[i].exec(core, p));
            if i == 0 && pgid.as_raw() == 0 { // 最初のexecが終わったら、pgidにコマンドのPIDを記録
                pgid = pids[0].expect("SUSHI INTERNAL ERROR (unforked in pipeline)");
            }
            prev = p.recv;
        }

        pids.push(
            self.commands[self.pipes.len()].exec(core, &mut Pipe::end(prev, pgid))
        );

        pids
    }

    pub fn new() -> Pipeline {
        Pipeline {
            text: String::new(),
            commands: vec![],
            pipes: vec![],
        }
    }

    fn eat_command(feeder: &mut Feeder, ans: &mut Pipeline, core: &mut ShellCore) -> bool {
        if let Some(command) = command::parse(feeder, core){
            ans.text += &command.get_text();
            ans.commands.push(command);

            let blank_len = feeder.scanner_blank(core);
            ans.text += &feeder.consume(blank_len);
            true
        }else{
            false
        }
    }

    fn eat_pipe(feeder: &mut Feeder, ans: &mut Pipeline, core: &mut ShellCore) -> bool {
        if let Some(p) = Pipe::parse(feeder, core) {
            ans.text += &p.text.clone();
            ans.pipes.push(p);
            true
        }else{
            false
        }
    }

    fn eat_blank_and_comment(feeder: &mut Feeder, ans: &mut Pipeline, core: &mut ShellCore) {
        loop {
            let blank_len = feeder.scanner_multiline_blank(core);
            ans.text += &feeder.consume(blank_len);             //空白、空行を削除
            let comment_len = feeder.scanner_comment();
            ans.text += &feeder.consume(comment_len);             //コメントを削除
            if blank_len + comment_len == 0 { //空白、空行、コメントがなければ出る
                break;
            }
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Pipeline> {
        let mut ans = Pipeline::new();

        if ! Self::eat_command(feeder, &mut ans, core){      //最初のコマンド
            return None;
        }

        while Self::eat_pipe(feeder, &mut ans, core){
            loop {
                Self::eat_blank_and_comment(feeder, &mut ans, core);
                if Self::eat_command(feeder, &mut ans, core) {
                    break; //コマンドがあれば73行目のloopを抜けてパイプを探す
                }
                if feeder.len() != 0 || ! feeder.feed_additional_line(core) {
                    return None;
                }
            }
        }

        Some(ans)
    }
}
