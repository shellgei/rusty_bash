//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::scanner::*;

pub struct Redirect {
    pub text: String,
    pub pos: DebugInfo,
    pub left_fd: i32,
    pub direction_str: String,
    pub path: String,
}

impl Redirect {
    pub fn parse(text: &mut Feeder) -> Option<Redirect> {
        if let Some(r) = and_arrow_redirect(text){
            return Some(r);
        }
    
        number_arrow_redirect(text)
    }
}

fn and_arrow_redirect(text: &mut Feeder) -> Option<Redirect> {
    if text.len() < 3 {
        return None;
    };

    if text.compare(0, "&>") || text.compare(0, ">&") {
        /* extract the file name */
        let start = scanner_while(text, 2, " ");
        let end = scanner_until_escape(text, start, " \t\n;");
        let path = text.from_to(start, end);

        Some( Redirect {
            text: text.consume(end),
            pos: DebugInfo::init(text),
            left_fd: -1,
            direction_str: "&>".to_string(),
            path: path,
        })
    }else{
        None
    }
}

/* > < 2> 0< 1> */
fn number_arrow_redirect(text: &mut Feeder) -> Option<Redirect> {
    let arrow_pos = scanner_until(text, 0, "<>");

    if text.len() < arrow_pos+1 {
        return None;
    };

    let mut fd;
    let dir;
    if text.nth(arrow_pos) == '<' {
        fd = 0;
        dir = '<'.to_string();
    }else if text.nth(arrow_pos) == '>' {
        fd = 1;
        dir = '>'.to_string();
    }else{
        return None;
    };

    /* read the number before the arrow */
    if arrow_pos != 0 {
        if let Ok(num) = text.from_to(0, arrow_pos).parse::<i32>() {
            fd = num;
        }else{
            return None;
        }
    };

    /* extract the file name */
    let start = scanner_while(text, arrow_pos+1, " ");
    let end = scanner_until_escape(text, start, ") \t\n");
    let path = text.from_to(start, end);

    Some( Redirect {
        text: text.consume(end),
        pos: DebugInfo::init(text),
        left_fd: fd,
        direction_str: dir,
        path: path,
    })
}
