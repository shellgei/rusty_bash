//SPDX-FileCopyrightText: 2026 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub(super) fn history_for_readline(core: &mut ShellCore) -> sushline::readline::History {
    let mut history = core
        .db
        .get_param("HISTFILE")
        .ok()
        .filter(|filename| !filename.is_empty())
        .and_then(|filename| sushline::readline::History::read_file(filename).ok())
        .unwrap_or_default();
    history.enforce_max_len(history_file_size_limit(core));
    for entry in core.history.iter().rev() {
        if !entry.is_empty() {
            history.push(entry.clone());
        }
    }
    history
}

fn history_file_size_limit(core: &mut ShellCore) -> Option<usize> {
    core.db
        .get_param("HISTFILESIZE")
        .ok()
        .and_then(|value| value.parse::<isize>().ok())
        .and_then(|value| (value >= 0).then_some(value as usize))
}

pub(super) fn update_current_history_entry(core: &mut ShellCore, extend: bool, line: &str) {
    if extend {
        if let Some(entry) = core.history.first_mut() {
            if !entry.is_empty() {
                entry.push('\n');
            }
            entry.push_str(line);
        }
    } else if let Some(entry) = core.history.first_mut() {
        *entry = line.to_string();
    }
}

pub(super) fn remove_current_history_entry(core: &mut ShellCore) {
    if !core.history.is_empty() {
        core.history.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ps2_line_extends_current_history_entry() {
        let mut core = ShellCore::new();
        core.history.push("echo \"alpha".to_string());
        update_current_history_entry(&mut core, true, "beta\"");
        assert_eq!(core.history[0], "echo \"alpha\nbeta\"");
    }
}
