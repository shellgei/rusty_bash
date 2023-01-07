//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;

pub struct Script {
    pub jobs: Vec<Job>,
    pub text: String,
}
