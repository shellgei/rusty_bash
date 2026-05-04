//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-FileCopyrightText: 2026 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

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
            .unwrap_or_default()
            .parse::<usize>()
        {
            file_line %= n;
        }

        sushline::readline::History::read_file(self.db.get_param("HISTFILE").unwrap_or_default())
            .ok()
            .and_then(|history| {
                history
                    .entries()
                    .iter()
                    .rev()
                    .nth(file_line)
                    .map(|entry| entry.line().into_owned())
            })
            .unwrap_or_default()
    }

    pub fn write_history_to_file(&mut self) {
        if !self.db.flags.contains('i') || self.is_subshell {
            return;
        }
        let filename = self.db.get_param("HISTFILE").unwrap_or_default();
        if filename.is_empty() {
            eprintln!("sush: HISTFILE is not set");
            return;
        }

        let file = match OpenOptions::new().create(true).append(true).open(&filename) {
            Ok(f) => f,
            _ => {
                eprintln!("sush: invalid history file");
                return;
            }
        };

        let mut f = BufWriter::new(file);
        for h in self.history.iter().rev() {
            if h.is_empty() {
                continue;
            }
            let _ = writeln!(f, "{h}");
        }
        let _ = f.flush();
    }
}
