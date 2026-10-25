//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::Pipe;
use crate::elements::command::Command;
use crate::elements::command::paren::ParenCommand;
use crate::elements::subword::Subword;
use crate::elements::word::mode::WordMode;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};
use nix::unistd;
use std::thread;
use nix::sys::wait;
use nix::sys::wait::WaitStatus;
use nix::sys::wait::WaitStatus::Signaled;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct ProcessSubstitution {
    pub text: String,
    command: ParenCommand,
    pub direction: char,
    pipe: Option<Pipe>,
}

impl Subword for ProcessSubstitution {
    fn get_text(&self) -> &str {
        self.text.as_ref()
    }
    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.direction == '>' {
            return self.substitute_out(core);
        }

        let mut pipe = Pipe::new("|".to_string());
        pipe.set(-1, unistd::getpgrp(), core)?;
        let pid = self.command.exec(core, &mut pipe)?.unwrap();
        //let pid_u: i32 = pid.into();
        /*
        core.db.last_bg_pid = pid.into();
        core.db.last_bg_exit_status = None;
        */
       // let bg_info = ((pid_u as u64) << 32) + 1000;
        //core.db.last_bg_proc.store(bg_info, Relaxed);
        core.out_proc_sub_pid.push(pid);
        self.text = "/dev/fd/".to_owned() + &pipe.recv.to_string();

        {
            let mut bg_info = core.db.last_bg_info.lock().unwrap();
            *bg_info = (Some(pid), None);
        }

        let proc_info = Arc::clone(&core.db.last_bg_info);

        thread::spawn(move || match wait::waitpid(pid, None) {
            Ok(WaitStatus::Exited(_, es)) => {
                let mut bg_info = proc_info.lock().unwrap();
                if let Some(p) = bg_info.0 {
                    if p == pid {
                        *bg_info = (Some(pid), Some(es));
                    }
                }
                //let bg_info = ((pid as u64) << 32) + 1000;
                //proc_info.store(
                //dbg!("{:?}", &pid);
                //dbg!("{:?}", &es);
       //         exit_status.store(es, Relaxed);
            }
            Ok(Signaled(pid, sig, _)) => {
                let mut bg_info = proc_info.lock().unwrap();
                if let Some(p) = bg_info.0 {
                    if p == pid {
                        *bg_info = (Some(pid), Some(sig as i32 + 128));
                    }
                }
        //        let _ = signal::killpg(pid, sig);
         //       exit_status.store(sig as i32 + 128, Relaxed);
            }
            Err(_) => {}
            _ => {}
        });

        Ok(())
    }

    fn set_pipe(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.direction == '>' {
            self.pipe = Some(Pipe::new(">()".to_string()));
            self.pipe
                .as_mut()
                .unwrap()
                .set(-1, unistd::getpgrp(), core)?;
        }
        Ok(())
    }

    fn is_to_proc_sub(&self) -> bool {
        self.text.starts_with(">(")
    }
}

impl ProcessSubstitution {
    fn substitute_out(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let pipe = self.pipe.as_mut().unwrap();
        let pid = self.command.exec(core, pipe)?.unwrap();
        core.out_proc_sub_pid.push(pid);
        core.out_proc_sub_fd.push((pipe.proc_sub_send, core.source_function_level));
        self.text = "/dev/fd/".to_owned() + &pipe.proc_sub_send.to_string();

        Ok(())
    }

    pub fn parse(
        feeder: &mut Feeder,
        core: &mut ShellCore,
        mode: &Option<WordMode>,
    ) -> Result<Option<Self>, ParseError> {
        if let Some(WordMode::Arithmetic) = mode {
            return Ok(None);
        }

        if !feeder.starts_with("<(") && !feeder.starts_with(">(") {
            return Ok(None);
        }
        let mut ans = ProcessSubstitution::default();
        ans.text = feeder.consume(1);
        ans.direction = ans.text.chars().nth(0).unwrap();

        if let Some(pc) = ParenCommand::parse(feeder, core, true)? {
            ans.text += &pc.get_text();
            ans.command = pc;
            return Ok(Some(ans));
        }

        Ok(None)
    }
}
