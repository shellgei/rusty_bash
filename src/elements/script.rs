//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;

#[derive(Debug)]
pub struct Script {
    pub jobs: Vec<Job>,
    pub job_ends: Vec<String>,
    pub text: String,
}
