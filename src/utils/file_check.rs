//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::path::Path;

pub fn is_regular_file(name: &str) -> bool {
    Path::new(name).is_file()
}
