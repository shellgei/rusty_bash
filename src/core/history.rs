//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::exec::ExecError;
use rev_lines::RevLines;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::fs::OpenOptions;

impl ShellCore {
    pub fn fetch_history(&mut self, pos: usize, prev: usize, prev_str: String) -> Result<String, ExecError> {
        if prev < self.history.len() {
            self.history[prev] = prev_str;
        }else{
            self.rewritten_history.insert(prev + 1 - self.history.len(), prev_str);
        }

        let ans = if pos < self.history.len() {
            self.history[pos].clone()
        }else{
            self.fetch_history_file(pos + 1 - self.history.len())?
        };

        Ok(ans)
    }

    pub fn fetch_history_file(&mut self, pos: usize) -> Result<String, ExecError> {
        if let Some(s) = self.rewritten_history.get(&pos) {
            return Ok(s.to_string());
        }
        if pos == 0 {
            return Ok(String::new());
        }

        let mut file_line = pos - 1;
        if let Ok(n) = self.db.get_param("HISTFILESIZE")?.parse::<usize>() {
            file_line %= n;
        }

        if let Ok(hist_file) = File::open(self.db.get_param("HISTFILE")?){
            let mut rev_lines = RevLines::new(BufReader::new(hist_file));
            if let Some(Ok(s)) = rev_lines.nth(file_line) {
                return Ok(s);
            }
        }

        Ok(String::new())
    }

    pub fn write_history_to_file(&mut self) -> Result<(), ExecError> {
        if ! self.flags.contains('i') || self.is_subshell {
            return Ok(());
        }
        let filename = self.db.get_param("HISTFILE")?;
        if filename.is_empty() {
            return Err(ExecError::Other("sush: HISTFILE is not set".to_string()));
        }
    
        let file = match OpenOptions::new().create(true)
                         .append(true).open(&filename) {
            Ok(f) => f,
            _     => {
                return Err(ExecError::Other("sush: invalid history file".to_string()));
            },
        };
    
        let mut f = BufWriter::new(file);
        for h in self.history.iter().rev() {
            if h.is_empty() {
                continue;
            }
            let _ = f.write(h.as_bytes());
            let _ = f.write(&[0x0A]);
        }
        let _ = f.flush();
        Ok(())
    }
}
