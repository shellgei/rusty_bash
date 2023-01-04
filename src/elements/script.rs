//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;
use crate::{Feeder, ShellCore};

pub struct Script {
    pub list: Vec<Job>,
    pub text: String,
}

impl Script {
    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        if let Some(job) = Job::parse(text, core){
            return Some( Script{text: job.text.clone(), list: vec!(job)} );
        }
        None
    }
}
