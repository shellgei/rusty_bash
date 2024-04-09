//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::pipeline::Pipeline;

#[derive(Debug)]
pub struct Job {
    pub pipelines: Vec<Pipeline>,
    pub pipeline_ends: Vec<String>,
    pub text: String,
}
