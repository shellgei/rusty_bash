//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::ShellCore;
use crate::scanner::*;
use crate::element_list::RedirectOp;
use crate::elem_arg::Arg;

pub struct Redirect {
    pub text: String,
    pub pos: DebugInfo,
    pub left_fd: i32,
    pub right_fd: i32,
    pub redirect_type: RedirectOp,
    pub path: String,
    pub right_arg: Option<Arg>,
}

impl Redirect {
    fn new(text :&Feeder) -> Redirect {
        Redirect {
            text: String::new(),
            pos: DebugInfo::init(text),
            left_fd: -1,
            right_fd: -1,
            redirect_type: RedirectOp::NoRedirect,
            path: String::new(),
            right_arg: None,
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Redirect> {
        let mut ans = Redirect::new(text);
        let backup = text.clone();
        let pos = scanner_number(text, 0);
        if pos > 0 {
            if let Ok(num) = text.from_to(0, pos).parse::<i32>() {
                ans.left_fd = num;
                ans.text += &text.consume(pos);
            }else{
                return None;
            }
        }

        let (pos, red) = scanner_redirect(text);
        if pos == 0 {
            text.rewind(backup);
            return None;
        }

        ans.redirect_type = red.unwrap();
        ans.text += &text.consume(pos);
        ans.text += &text.consume_blank();

        if ans.left_fd == -1 {
            if ans.redirect_type == RedirectOp::Input {
                ans.left_fd = 0;
            }else if ans.redirect_type == RedirectOp::Output {
                ans.left_fd = 1;
            }
        }


        if let Some(a) = Arg::parse(text, conf, false, false) {
            ans.text += &a.text.clone();
            ans.right_arg = Some(a);
        }else{
            text.rewind(backup);
            return None;
        };


        Some(ans)
    }
}

/*
fn and_arrow_redirect(text: &mut Feeder) -> Option<Redirect> {
    if text.len() < 3 {
        return None;
    };

    if text.starts_with( "&>") || text.starts_with( ">&") {
        /* extract the file name */
        let start = scanner_blank(text, 2);
        let end = scanner_until_escape(text, start, " \t\n;");
        let path = text.from_to(start, end);

        Some( Redirect {
            text: text.consume(end),
            pos: DebugInfo::init(text),
            left_fd: -1,
            right_fd: -1,
            redirect_type: RedirectOp::AndOutput /*"&>".to_string()*/,
            path: path,
            right_arg: None,
        })
    }else{
        None
    }
}
    */

    /*
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
        dir = RedirectOp::Input /*'<'.to_string()*/;
    }else if text.nth(arrow_pos) == '>' {
        fd = 1;
        dir = RedirectOp::Output /*'>'.to_string()*/;
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
    let start = scanner_blank(text, arrow_pos+1);
    let end = scanner_until_escape(text, start, ") \t\n");
    let path = text.from_to(start, end);

    Some( Redirect {
        text: text.consume(end),
        pos: DebugInfo::init(text),
        left_fd: fd,
        right_fd: -1,
        redirect_type: dir,
        path: path,
        right_arg: None,
    })
}

*/
