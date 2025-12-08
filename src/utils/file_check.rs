//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

extern crate libc;
use libc::isatty;
use faccess;
use faccess::PathExt;
use nix::unistd;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};

#[cfg(target_os = "android")]
use std::os::android::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;
use std::os::unix::fs::MetadataExt as UnixMetadataExt;

use std::path::Path;

pub fn exists(name: &str) -> bool {
    if name.ends_with("/") {
        //for macOS
        return is_dir(name);
    }

    fs::metadata(name).is_ok()
}

pub fn is_regular_file(name: &str) -> bool {
    Path::new(name).is_file()
}

pub fn is_dir(name: &str) -> bool {
    Path::new(name).is_dir()
}

pub fn metadata_comp(left: &str, right: &str, tp: &str) -> bool {
    let (lmeta, rmeta) = match (fs::metadata(left), fs::metadata(right)) {
        (Ok(lm), Ok(rm)) => (lm, rm),
        (Ok(_), Err(_)) => return tp == "-nt",
        (Err(_), Ok(_)) => return tp == "-ot",
        (Err(_), Err(_)) => return false,
    };

    match tp {
        "-ef" => (lmeta.dev(), lmeta.ino()) == (rmeta.dev(), rmeta.ino()),
        "-nt" => lmeta.modified().unwrap() > rmeta.modified().unwrap(),
        "-ot" => lmeta.modified().unwrap() < rmeta.modified().unwrap(),
        _ => false,
    }
}

pub fn metadata_check(name: &str, tp: &str) -> bool {
    let meta = match fs::metadata(name) {
        Ok(m) => m,
        _ => return false,
    };

    match tp {
        "-b" => return meta.file_type().is_block_device(),
        "-c" => return meta.file_type().is_char_device(),
        "-p" => return meta.file_type().is_fifo(),
        "-s" => return meta.len() == 0,
        "-G" => return unistd::getgid() == meta.st_gid().into(),
        "-N" => {
            let modified_time = match meta.modified() {
                Ok(t) => t,
                _ => return false,
            };
            let accessed_time = match meta.accessed() {
                Ok(t) => t,
                _ => return false,
            };
            return modified_time > accessed_time;
        }
        "-O" => return unistd::getuid() == meta.st_uid().into(),
        "-S" => return meta.file_type().is_socket(),
        _ => {}
    }

    let special_mode = (meta.permissions().mode() / 0o1000) % 8;
    match tp {
        "-g" => (special_mode % 4) >> 1 == 1,
        "-k" => special_mode % 2 == 1,
        "-u" => special_mode / 4 == 1,
        _ => false,
    }
}

pub fn is_symlink(name: &str) -> bool {
    Path::new(name).is_symlink()
}

pub fn is_readable(name: &str) -> bool {
    Path::new(&name).readable()
}

pub fn is_executable(name: &str) -> bool {
    Path::new(name).executable()
}

pub fn is_writable(name: &str) -> bool {
    Path::new(&name).writable()
}

pub fn is_tty(name: &str) -> bool {
    let fd = match name.parse::<i32>() {
        Ok(n) => n,
        _ => return false,
    };
    unsafe{isatty(fd) == 0}
    //unistd::isatty(RawFd::from(fd)) == Ok(true)
}
