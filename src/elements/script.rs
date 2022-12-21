//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::pipeline::Pipeline;

pub struct Script {
    pub list: Vec<Pipeline>,
    pub text: String,
}
