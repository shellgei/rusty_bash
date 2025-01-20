//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod exec;
pub mod input;
pub mod parse;

/*
pub fn print(s: &str, core: &mut ShellCore) {
    let name = core.db.get_param("0").unwrap();
    if core.flags.contains('i') {
        eprintln!("{}: {}", &name, &s);
    }else{
        let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
        eprintln!("{}: line {}: {}", &name, &lineno, s);
    }
}

pub fn internal(s: &str) -> String {
    format!("SUSH INTERNAL ERROR: {}", s)
}

pub fn exponent(s: &str) -> String {
    format!("exponent less than 0 (error token is \"{}\")", s)
}

/* error at wait */
pub fn signaled(pid: Pid, signal: Signal, coredump: bool) -> i32 {
    match coredump {
        true  => eprintln!("Pid: {:?}, Signal: {:?} (core dumped)", pid, signal),
        false => eprintln!("Pid: {:?}, Signal: {:?}", pid, signal),
    }
    128+signal as i32
}
*/
