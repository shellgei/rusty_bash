//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::element_list::{ControlOperator, RedirectOp/*, Reserved*/};

pub fn scanner_until_escape(text: &Feeder, from: usize, to: &str) -> usize {
    let mut pos = from;
    let mut escaped = false;
    for ch in text.chars_after(from) {
        if escaped || ch == '\\' {
            escaped = !escaped;
        }else if let Some(_) = to.find(ch) {
            break;
        };
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_blank(text: &Feeder, from: usize) -> usize {
    let mut pos = from;
    for ch in text.chars_after(from) {
        if let Some(_) = " \t".find(ch) {
            pos += ch.len_utf8();
        }else{
            break;
        };
    }
    pos
}

pub fn scanner_until(text: &Feeder, from: usize, to: &str) -> usize {
    let mut pos = from;
    for ch in text.chars_after(from) {
        if let Some(_) = to.find(ch) {
            break;
        };
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_name_or_parameter(text: &Feeder, from: usize) -> usize {
    let ans = scanner_parameter(text, from);

    if ans == 0 {
        scanner_name(text, from)
    }else{
        ans
    }
}

pub fn scanner_number(text: &Feeder, from: usize) -> usize {
    let mut pos = from;
    for ch in text.chars_after(from) {
        if ch < '0' || '9' < ch {
            break;
        }
        pos += 1;
    }
    pos
}

pub fn scanner_parameter(text: &Feeder, from: usize) -> usize {
    if text.len() < from {
        return from;
    }

    if "?*@$#!-:".chars().any(|c| c == text.nth(from)) { //special parameters
        return from+1;
    };

    scanner_number(text, from)
}

/*
pub enum Redirect {
    Output, /* > */ 
    Input, /* < */
    InOut, /* <> */
    AndOutput, /* &> */ 
    OutputAnd, /* >& */ 
    Append, /* >> */ 
    HereDoc, /* << */ 
    AndAppend, /* &>> */ 
    HereStr, /* <<< */ 
}*/

pub fn scanner_redirect(text: &Feeder) -> (usize, Option<RedirectOp> ) {
    if text.starts_with("<<<") {
        return (3, Some(RedirectOp::HereStr));
    }else if text.starts_with("&>>") {
        return (3, Some(RedirectOp::AndAppend));
    }else if text.starts_with(">>") {
        return (3, Some(RedirectOp::Append));
    }else if text.starts_with("<<") {
        return (3, Some(RedirectOp::HereDoc));
    }else if text.starts_with(">&") {
        return (3, Some(RedirectOp::OutputAnd));
    }else if text.starts_with("&>") {
        return (3, Some(RedirectOp::AndOutput));
    }else if text.starts_with("<>") {
        return (3, Some(RedirectOp::InOut));
    }else if text.starts_with(">") {
        return (3, Some(RedirectOp::Output));
    }else if text.starts_with("<") {
        return (3, Some(RedirectOp::Input));
    }
    (0, None)
}

pub fn scanner_name(text: &Feeder, from: usize) -> usize {
    if text.len() <= from {
        return from;
    }

    let h = &text.nth(0);
    if !((*h >= 'A' && *h <= 'Z') || (*h >= 'a' && *h <= 'z') || *h == '_') {
        return from;
    }

    if text.len() == from+1 {
        return from+1;
    }

    let mut ans = from+1;
    for c in text.chars_after(from+1) {
        if !((c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') 
        || (c >= 'a' && c <= 'z') || c == '_') {
            break;
        }
        ans += 1;
    }

    return ans;
}

/*
pub fn scanner_reserved(text: &Feeder) -> (usize, Option<Reserved> ) {
    if text.starts_with("function"){
        return (8, Some(Reserved::Function));
    }

    (0, None)
}
*/

pub fn scanner_control_op(text: &Feeder) -> (usize, Option<ControlOperator> ) {
    let mut op = None;
    let mut pos = 0;

    if text.len() > 2  {
        pos = 3;
        op = if text.starts_with(";;&") {
            Some(ControlOperator::SemiSemiAnd)
        }else{
            None
        };
    }

    if op == None && text.len() > 1  {
        pos = 2;
        op = if text.starts_with("||") {
            Some(ControlOperator::Or)
        }else if text.starts_with("&&") {
            Some(ControlOperator::And)
        }else if text.starts_with(";;") {
            Some(ControlOperator::DoubleSemicolon)
        }else if text.starts_with(";&") {
            Some(ControlOperator::SemiAnd)
        }else if text.starts_with("|&") {
            Some(ControlOperator::PipeAnd)
        }else{
            None
        };

    }

    if op == None && text.len() > 0  {
        pos = 1;
        if text.starts_with("&") {
            if text.len() > 1 && text.nth(1) == '>' {
                return (0, None)
            }
            return (1, Some(ControlOperator::BgAnd));
        } else if text.starts_with("\n") {
            return (1, Some(ControlOperator::NewLine));
        } else if text.starts_with("|") {
            return (1, Some(ControlOperator::Pipe));
        } else if text.starts_with(";") {
            return (1, Some(ControlOperator::Semicolon));
        } else if text.starts_with("(") {
            return (1, Some(ControlOperator::LeftParen));
        } else if text.starts_with(")") {
            return (1, Some(ControlOperator::RightParen));
        }
    }

    if op != None && text.len() > pos && text.nth(pos) == '\n' {
        pos += 1;
    }

    if op != None{
        return (pos, op);
    }


    (0 , None)
}

pub fn scanner_comment(text: &Feeder, from: usize) -> usize {
    if text.len() > from && text.nth_is(from, "#") {
        return scanner_until(text, from, "\n");
    }

    from
}

pub fn scanner_end_paren(text: &Feeder, from: usize) -> usize {
    if text.len() == 0 {
        return 0;
    }

    if text.nth_is(from, ")") {
        return from+1;
    }
    return from;
}

/* TODO: these scanners should be summarized. */ 
pub fn scanner_start_paren(text: &Feeder, from: usize) -> usize {
    if text.len() == 0 {
        return 0;
    }

    if text.nth_is(from, "(") {
        return from+1;
    }
    return from;
}

pub fn scanner_start_brace(text: &Feeder, from: usize) -> usize {
    if text.len() == 0 {
        return 0;
    }

    if text.nth_is(from, "{") {
        return from+1;
    }
    return from;
}

pub fn scanner_integer(text: &Feeder, from: usize) -> usize {
    if text.len() == from {
        return from;
    }

    let mut pos = from;
    if text.nth(from) == '-' {
        pos += 1;
    }

    for ch in text.chars_after(pos) {
        if ch < '0' || ch > '9' {
            break;
        }

        pos += 1;
    }

    if text.nth(from) == '-' && pos == from+1 {
        from
    }else{
        pos
    }
}
