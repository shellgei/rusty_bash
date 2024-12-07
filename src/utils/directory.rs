use std::fs::DirEntry;
use std::path::Path;
use super::glob;

pub fn files(dir: &str) -> Vec<String> {
    let dir = if dir == "" {"."}else{dir};

    let entries = match Path::new(dir).read_dir() {
        Ok(es) => es,
        Err(_) => return vec![],
    };

    let f = |e: DirEntry| e.file_name()
               .to_string_lossy().to_string();

    entries.map(|e| f(e.unwrap()) ).collect()
}

pub fn glob(dir: &str, pattern: &str) -> Vec<String> {
    let make_path = |file| dir.to_owned() + file + "/";

    let mut fs = files(dir);
    let pat = glob::parse(pattern);
    fs.retain(|f| glob::compare(f, &pat));

    fs.iter().map(|f| make_path(&f) ).collect()
}
