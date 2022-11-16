//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::process::exit;

impl ShellCore {
    pub fn exit(&mut self, args: &mut Vec<String>) -> i32 {
        if args.len() >= 2 {
            if let Ok(status) = args[1].parse::<i32>(){
                exit(status);
            }else{
                eprintln!("exit: {}: numeric argument required", args[1]);
                exit(2);
            }
        }

        if let Ok(status) = self.get_var("?").to_string().parse::<i32>(){
            exit(status);
        }else{
            eprintln!("Shell internal error");
            exit(1);
        }
    }
}
