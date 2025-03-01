//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{file_check, Feeder, ShellCore, utils};
use crate::core::builtins::completion;
use crate::error::exec::ExecError;
use crate::elements::command::simple::SimpleCommand;
use crate::elements::command::Command;
use crate::elements::io::pipe::Pipe;
use crate::feeder::terminal::Terminal;
use unicode_width::UnicodeWidthStr;

fn str_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

fn common_length(chars: &Vec<char>, s: &String) -> usize {
    let max_len = chars.len();
    for (i, c) in s.chars().enumerate() {
        if i >= max_len || chars[i] != c {
            return i;
        }
    }
    max_len
}

fn common_string(paths: &Vec<String>) -> String {
    if paths.is_empty() {
        return "".to_string();
    }

    let ref_chars: Vec<char> = paths[0].chars().collect();
    let mut common_len = ref_chars.len();

    for path in &paths[1..] {
        let len = common_length(&ref_chars, &path);
        common_len = std::cmp::min(common_len, len);
    }

    ref_chars[..common_len].iter().collect()
}

fn is_dir(s: &str, core: &mut ShellCore) -> bool {
    let tilde_prefix = "~/".to_string();
    let tilde_path = core.db.get_param("HOME").unwrap_or(String::new()) + "/";

    file_check::is_dir(&s.replace(&tilde_prefix, &tilde_path))
}

impl Terminal {
    pub fn completion(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        self.escape_at_completion = true;
        let _ = core.db.set_array("COMPREPLY", vec![], None);
        self.set_completion_info(core)?;

        if ! self.set_custom_compreply(core).is_ok()
        && ! self.set_default_compreply(core).is_ok() {
            self.cloop();
            return Ok(());
        }

        match self.tab_num  {
            1 => self.try_completion(core).unwrap(),
            _ => self.show_list(&core.db.get_array_all("COMPREPLY")),
        }
        Ok(())
    }

    fn exec_complete_function(org_word: &str, prev_pos: i32, cur_pos: i32, 
                              core: &mut ShellCore)-> Result<(), ExecError> {
        let prev_word = core.db.get_array_elem("COMP_WORDS", &prev_pos.to_string())?;
        let target_word = core.db.get_array_elem("COMP_WORDS", &cur_pos.to_string())?;
        let info = &core.current_completion_info;

        let command = format!("{} \"{}\" \"{}\" \"{}\"",
                                &info.function, &org_word, &target_word, &prev_word);
        let mut feeder = Feeder::new(&command);

        if let Ok(Some(mut a)) = SimpleCommand::parse(&mut feeder, core) {
            let mut dummy = Pipe::new("".to_string());
            a.exec(core, &mut dummy)?;
        }
        Ok(())
    }

    fn exec_action(cur_pos: i32, core: &mut ShellCore)-> Result<(), ExecError> {
        let target_word = core.db.get_array_elem("COMP_WORDS", &cur_pos.to_string())?;
        let info = &core.current_completion_info;

        let command = format!("COMPREPLY=($(compgen -A \"{}\" \"{}\"))",  &info.action, &target_word);
        let mut feeder = Feeder::new(&command);

        if let Ok(Some(mut a)) = SimpleCommand::parse(&mut feeder, core) {
            let mut dummy = Pipe::new("".to_string());
            a.exec(core, &mut dummy)?;
        }
        Ok(())
    }

    fn set_custom_compreply(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let cur_pos = Self::get_cur_pos(core);
        let prev_pos = cur_pos - 1;
        let word_num = core.db.len("COMP_WORDS") as i32;

        if prev_pos < 0 || prev_pos >= word_num {
            return Err(ExecError::Other("pos error".to_string()));
        }

        let org_word = core.db.get_array_elem("COMP_WORDS", "0")?;

        let info = core.completion_info.get(&org_word)
                   .ok_or(ExecError::Other("no completion function".to_string()))?;

        core.current_completion_info = info.clone();
        if info.function != "" {
            Self::exec_complete_function(&org_word, prev_pos, cur_pos, core)?;
        }else{
            Self::exec_action(cur_pos, core)?;
        }

        match core.db.len("COMPREPLY") {
            0 => Err(ExecError::Other("no completion cand".to_string())),
            _ => Ok(()),
        }
    }

    fn get_cur_pos(core: &mut ShellCore) -> i32 {
        core.db.get_param("COMP_CWORD").unwrap().parse::<i32>().unwrap()
    }

    pub fn set_default_compreply(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let pos = core.db.get_param("COMP_CWORD")?;
        let last = core.db.get_array_elem("COMP_WORDS", &pos)?;

        let com = core.db.get_array_elem("COMP_WORDS", "0")?;

        let (tilde_prefix, tilde_path, last_tilde_expanded) = Self::set_tilde_transform(&last, core);

        let mut args = vec!["".to_string(), "".to_string(), last_tilde_expanded.to_string()];

        let list = self.make_default_compreply(core, &mut args, &com, &pos);
        if list.is_empty() {
            return Err(ExecError::Other("empty list".to_string()));
        }

        let tmp: Vec<String> = list.iter().map(|p| p.replacen(&tilde_path, &tilde_prefix, 1)).collect();
        core.db.set_array("COMPREPLY", tmp, None)
    }

    fn make_default_compreply(&mut self, core: &mut ShellCore, args: &mut Vec<String>,
                              com: &str, pos: &str) -> Vec<String> {
        if core.completion_info.contains_key(com) {
            let action = core.completion_info[com].action.clone();
            let options = core.completion_info[com].options.clone();

            if action != "" {
                let mut cands = match action.as_ref() {
                    "alias" => completion::compgen_a(core, args),
                    "command" => completion::compgen_c(core, args),
                    "job" => completion::compgen_j(core, args),
                    "setopt" => completion::compgen_o(core, args),
                    "stopped" => completion::compgen_stopped(core, args),
                    "user" => completion::compgen_u(core, args),
                    "variable" => completion::compgen_v(core, args),
                    _ => vec![],
                };
    
                if options.contains_key("-P") {
                    let prefix = &options["-P"];
                    cands = cands.iter().map(|c| prefix.clone() + c).collect();
                }
                if options.contains_key("-S") {
                    let suffix = &options["-S"];
                    cands = cands.iter().map(|c| c.to_owned() + &suffix.clone()).collect();
                }
                return cands;
            }
        }

        if pos == "0" {
            return if core.db.len("COMP_WORDS") == 0 {
                self.escape_at_completion = false;
                completion::compgen_h(core, args).to_vec().into_iter().filter(|h| h.len() > 0).collect()
            }else{
                completion::compgen_c(core, args)
            };
        }

        completion::compgen_f(core, args)
    }

    pub fn try_completion(&mut self, core: &mut ShellCore) -> Result<(), String> {
        let pos = core.db.get_param("COMP_CWORD")?;
        let target = core.db.get_array_elem("COMP_WORDS", &pos)?;

        if core.db.len("COMPREPLY") == 1 {
            let arr = core.db.get_array_all("COMPREPLY");
            let output = arr[0].clone();
            let mut tail = match is_dir(&output, core) {
                true  => "/",
                false => " ",
            };

            if core.current_completion_info.o_options.contains(&"nospace".to_string()) {
                tail = "";
            }

            self.replace_input(&(output + tail));
            return Ok(());
        }

        let common = common_string(&core.db.get_array_all("COMPREPLY"));
        if common.len() != target.len() {
            self.replace_input(&common);
            return Ok(());
        }
        self.cloop();
        Ok(())
    }

    fn normalize_tab(&mut self, row_num: i32, col_num: i32) {
        let i = (self.tab_col*row_num + self.tab_row + row_num*col_num)%(row_num*col_num);
        self.tab_col = i/row_num;
        self.tab_row = i%row_num;
    }

    fn show_list(&mut self, list: &Vec<String>) {
        if list.is_empty() {
            return;
        }

        let widths: Vec<usize> = list.iter().map(|s| str_width(s)).collect();
        let max_entry_width = widths.iter().max().unwrap_or(&1000) + 1;
        let terminal_row_num = self.size.1;
        let col_num = std::cmp::min(
                          std::cmp::max(self.size.0 / max_entry_width, 1),
                          list.len()
                      );
        let row_num = std::cmp::min(
                          (list.len()-1) / col_num + 1,
                          std::cmp::max(terminal_row_num - 2, 1)
                      );
        self.completion_candidate = String::new();

        if self.tab_num > 2 {
            self.normalize_tab(row_num as i32, col_num as i32);
        }

        eprintln!("\r");
        for row in 0..row_num {
            for col in 0..col_num {
                let tab = self.tab_row == row as i32 && self.tab_col == col as i32;
                self.print_an_entry(list, &widths, row, col, 
                    row_num, max_entry_width, tab);
            }
            print!("\r\n");
        }

        let (cur_col, cur_row) = self.head_to_cursor_pos(self.head, self.prompt_row);

        self.check_scroll();
        match cur_row as usize == terminal_row_num {
            true => {
                let back_row = std::cmp::max(cur_row as i16 - row_num as i16, 1);
                self.write(&termion::cursor::Goto(cur_col as u16, back_row as u16).to_string());
                print!("\x1b[1A");
                self.flush();
            },
            false => self.rewrite(false),
        }
    }

    fn print_an_entry(&mut self, list: &Vec<String>, widths: &Vec<usize>,
        row: usize, col: usize, row_num: usize, width: usize, pointed: bool) {
        let i = col*row_num + row;
        let space_num = match i < list.len() {
            true  => width - widths[i],
            false => width,
        };
        let cand = match i < list.len() {
            true  => list[i].clone(),
            false => "".to_string(),
        };

        let s = String::from_utf8(vec![b' '; space_num]).unwrap();
        if pointed {
            print!("\x1b[01;7m{}{}\x1b[00m", &cand, &s);
            self.completion_candidate = cand;
        }else{
            print!("{}{}", &cand, &s);
        }
    }

    fn shave_existing_word(&mut self) {
        while self.head > self.prompt.chars().count() 
        && ( self.head > 0 && self.chars[self.head-1] != ' ' ||
           (self.head > 1 && self.chars[self.head-1] == ' ' 
            && self.chars[self.head-2] == '\\') ) {
            self.backspace();
        }
        while self.head < self.chars.len() 
        && self.chars[self.head] != ' ' {
            self.delete();
        }
    }

    pub fn replace_input(&mut self, to: &String) {
        self.shave_existing_word();

        let to_escaped = if to.ends_with(" ") {
            let mut tmp = to.to_string();
            tmp.pop();
            match self.escape_at_completion {
                true  => tmp.replace(" ", "\\ ") + " ",
                false => tmp.replace("↵ \0", "\n") + " ",
            }
        }else {
            match self.escape_at_completion {
                true  => to.replace(" ", "\\ ").to_string(),
                false => to.replace("↵ \0", "\n").to_string(),
            }
        };

        for c in to_escaped.chars() {
            self.insert(c);
            self.check_scroll();
        }

        if to.ends_with(" ") 
        && self.head < self.chars.len() 
        && self.chars[self.head] == ' ' {
            self.backspace();
        }

        self.rewrite(true);
    }

    fn set_tilde_transform(last: &str, core: &mut ShellCore) -> (String, String, String) {
        let tilde_prefix;
        let tilde_path;
        let last_tilde_expanded;

        if last.starts_with("~/") {
            tilde_prefix = "~/".to_string();
            tilde_path = core.db.get_param("HOME").unwrap_or(String::new()) + "/";
            last_tilde_expanded = last.replacen(&tilde_prefix, &tilde_path, 1);
        }else{
            tilde_prefix = String::new();
            tilde_path = String::new();
            last_tilde_expanded = last.to_string();
        }

        (tilde_prefix, tilde_path, last_tilde_expanded)
    }

    fn set_completion_info(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let prompt_len = self.prompt.chars().count();
        core.db.set_param("COMP_POINT", &(self.head - prompt_len).to_string(), None)?;

        let all_string = self.get_string(prompt_len);
        core.db.set_param("COMP_LINE", &all_string, None)?;

        let tp = match self.tab_num {
            1 => "\t",
            _ => "?",
        };
        core.db.set_param("COMP_TYPE", &tp, None)?;
        core.db.set_param("COMP_KEY", "9", None)?;

        let mut words_all = utils::split_words(&all_string);
        //words_all.retain(|e| e != "");

        let left_string: String = self.chars[prompt_len..self.head].iter().collect();
        let mut words_left = utils::split_words(&left_string);
        //words_left.retain(|e| e != "");
        let from = completion_from(&words_left, core);

        words_all = words_all[from..].to_vec();
        words_left = words_left[from..].to_vec();
        let _ = core.db.set_array("COMP_WORDS", words_all, None);

        let mut num = words_left.len();
        match left_string.chars().last() {
            Some(' ') => {num -= 1},
            Some(_) => {
                if num > 0 {
                    num -= 1
                }
            },
            _ => {},
        }

        let _ = core.db.set_param("COMP_CWORD", &num.to_string(), None);
        Ok(())
    }
}

fn completion_from(ws: &Vec<String>, core: &mut ShellCore) -> usize {
    for i in 0..ws.len() {
        if utils::reserved(&ws[i]) {
            continue;
        }

        let s = ws[i..].join(" ");
        let mut feeder = Feeder::new(&s);
        if let Ok(Some(_)) = SimpleCommand::parse(&mut feeder, core) {
            return i;
        }
    }
    ws.len()
}
