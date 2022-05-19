//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use glob::glob;

pub fn eval_glob(globstr: &String) -> Vec<String> {
    let mut ans: Vec<String> = vec!();

    if let Ok(path) = glob(&globstr) {
        for dir in path {
            match dir {
                Ok(d) => {
                    if let Some(s) = d.to_str() {
                        ans.push(s.to_string());
                    };
                },
                _ => (),
            }
        };
    };
    ans
}
