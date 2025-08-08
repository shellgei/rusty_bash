//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::ShellCore;
use rev_lines::RevLines;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Write};

impl ShellCore {
    pub fn fetch_history(&mut self, pos: usize, prev: usize, prev_str: String) -> String {
        if prev < self.history.len() {
            self.history[prev] = prev_str;
        } else {
            self.rewritten_history
                .insert(prev + 1 - self.history.len(), prev_str);
        }

        if pos < self.history.len() {
            self.history[pos].clone()
        } else {
            self.fetch_history_file(pos + 1 - self.history.len())
        }
    }

    pub fn fetch_history_file(&mut self, pos: usize) -> String {
        if let Some(s) = self.rewritten_history.get(&pos) {
            return s.to_string();
        }
        if pos == 0 {
            return String::new();
        }

        let mut file_line = pos - 1;
        if let Ok(n) = self
            .db
            .get_param("HISTFILESIZE")
            .unwrap_or(String::new())
            .parse::<usize>()
        {
            file_line %= n;
        }

        if let Ok(hist_file) = File::open(self.db.get_param("HISTFILE").unwrap_or(String::new())) {
            let mut rev_lines = RevLines::new(BufReader::new(hist_file));
            if let Some(Ok(s)) = rev_lines.nth(file_line) {
                return s;
            }
        }

        String::new()
    }

    pub fn write_history_to_file(&mut self) {
        if !self.db.flags.contains('i') || self.is_subshell {
            return;
        }
        let filename = self.db.get_param("HISTFILE").unwrap_or(String::new());
        if filename == "" {
            eprintln!("sush: HISTFILE is not set");
            return;
        }

        let file = match OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&filename)
        {
            Ok(f) => f,
            _ => {
                eprintln!("sush: invalid history file");
                return;
            }
        };

        let mut f = BufWriter::new(file);
        for h in self.history.iter().rev() {
            if h == "" {
                continue;
            }
            let _ = f.write(h.as_bytes());
            let _ = f.write(&[0x0A]);
        }
        let _ = f.flush();
    }
}
