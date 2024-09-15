//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::elem;
use super::elem::Elem;

pub fn rearrange(elements: &[Elem]) -> Result<Vec<Elem>, Elem> {
    let mut ans = vec![];
    let mut stack = vec![];

    for e in elements {
        match e {
            Elem::BinaryOp(op) => match op.as_str() {
                "&&" | "||" => ans.push(Elem::Delimiter(op.to_string())),
                _ => {},
            },
            _ => {},
        }
        let ok = match e {
            Elem::Float(_) | Elem::Integer(_) | Elem::Word(_, _) | Elem::InParen(_)
                             => {ans.push(e.clone()); true},
            op               => rev_polish_op(&op, &mut stack, &mut ans),
        };

        if !ok {
            return Err(e.clone());
        }
    }

    while stack.len() > 0 {
        ans.push(stack.pop().unwrap());
    }

    Ok(ans)
}

fn rev_polish_op(elem: &Elem,
                 stack: &mut Vec<Elem>, ans: &mut Vec<Elem>) -> bool {
    loop {
        match stack.last() {
            None => {
                stack.push(elem.clone());
                break;
            },
            Some(_) => {
                let last = stack.last().unwrap();
                if elem::op_order(last) <= elem::op_order(elem) {
                    stack.push(elem.clone());
                    break;
                }
                ans.push(stack.pop().unwrap());
            },
        }
    }

    true
}
