//SPDX-FileCopyrightText: 2025 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use std::env;
use std::fs;
use std::collections::HashSet;
use std::os::unix::fs::PermissionsExt;
use rustyline::completion::Pair;

pub fn get_commands(prefix: &str) -> Vec<Pair> {
    let commands_set: HashSet<String> = env::var("PATH")
        .ok()
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| {
            let meta = entry.metadata().ok()?;
            if meta.is_file() && (meta.permissions().mode() & 0o111 != 0) {
                entry.file_name().into_string().ok().filter(|name| name.starts_with(prefix))
            } else {
                None
            }
        })
        .collect();

    let mut pairs: Vec<Pair> = commands_set
        .into_iter()
        .map(|name| Pair { display: name.clone(), replacement: name })
        .collect();
    pairs.sort_by(|a, b| a.display.cmp(&b.display));
    pairs
}