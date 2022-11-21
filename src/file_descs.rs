//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::{close, dup2};
use std::os::unix::prelude::RawFd;
use crate::elem_redirect::Redirect;
use crate::element_list::RedirectOp;
use std::fs::OpenOptions;
use std::os::unix::io::IntoRawFd;
use crate::ShellCore;

pub struct FileDescs {
    pub redirects: Vec<Box<Redirect>>,
    pub pipein: RawFd,
    pub pipeout: RawFd,
    pub prevpipein: RawFd,
}

impl FileDescs {
    pub fn new() -> FileDescs {
        FileDescs {
            redirects: vec![],
            pipein: -1,
            pipeout: -1,
            prevpipein: -1,
        }
    }

    pub fn no_connection(&self) -> bool {
        self.redirects.len() == 0  &&
            self.pipein == -1 &&
            self.pipeout == -1 &&
            self.prevpipein == -1
    }

    pub fn set_child_io(&mut self, conf: &mut ShellCore) -> Result<(), String> {
        if self.pipein != -1 {
            close(self.pipein).expect("Cannot close in-pipe");
        }
        if self.pipeout != -1 {
            Self::dup_and_close(self.pipeout, 1);
        }
    
        if self.prevpipein != -1 {
            Self::dup_and_close(self.prevpipein, 0);
        }
    
        for r in &mut self.redirects {
            if let Err(s) = Self::set_redirect(r, conf) {
                return Err(s);
            }
        };
    
        Ok(())
    }
    
    fn set_redirect(r: &mut Box<Redirect>, conf: &mut ShellCore) -> Result<(), String> {
        let path = r.eval(conf);
        if r.redirect_type == RedirectOp::Output /*">"*/ {
            if let Ok(file) = OpenOptions::new().truncate(true).write(true).create(true).open(&path){
                Self::dup_and_close(file.into_raw_fd(), r.left_fd);
            }else{
                panic!("Cannot open the file: {}", path);
            };
        }else if r.redirect_type == RedirectOp::OutputAnd  {
            if let Ok(n) = path.parse::<i32>() {
                dup2(n, r.left_fd).expect("Invalid fd");
            }else{
                conf.set_var("?", "1");
                if let Some(a) = &r.right_arg {
                    return Err(format!("bash: {}: ambiguous redirect", a.text.clone()));
                }else{
                    return Err("Unknown redirect error".to_string());
                }
            }
        }else if r.redirect_type == RedirectOp::AndOutput {
            if let Ok(file) = OpenOptions::new().truncate(true).write(true).create(true).open(&path){
                Self::dup_and_close(file.into_raw_fd(), 1);
                dup2(1, 2).expect("Redirection error on &>");
            }else{
                panic!("Cannot open the file: {}", path);
            };
        }else if r.redirect_type == RedirectOp::Input /*"<"*/ {
            if let Ok(file) = OpenOptions::new().read(true).open(&path){
                Self::dup_and_close(file.into_raw_fd(), r.left_fd);
            }else{
                panic!("Cannot open the file: {}", path);
            };
        }
    
        Ok(())
    }

    pub fn dup_and_close(from: RawFd, to: RawFd){
        close(to).expect(&("Can't close fd: ".to_owned() + &to.to_string()));
        dup2(from, to).expect("Can't copy file descriptors");
        close(from).expect(&("Can't close fd: ".to_owned() + &from.to_string()));
    }
    
    
    pub fn set_parent_io(pout: RawFd) {
        if pout >= 0 {
            close(pout).expect("Cannot close outfd");
        };
    }
}
