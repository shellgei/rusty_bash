//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use super::elem::ArithElem;

pub fn rearrange(elements: &[ArithElem]) -> Result<Vec<ArithElem>, ExecError> {
    let mut ans = vec![];
    let mut stack: Vec<ArithElem> = vec![];

    for e in elements {
        let ok = match e {
            ArithElem::Float(_) | ArithElem::Integer(_) |
            ArithElem::Variable(_, _, _) | ArithElem::InParen(_)
                             => { ans.push(e.clone()); true },
            op               => rev_polish_op(&op, &mut stack, &mut ans),
        };

        if !ok {
            return Err(ExecError::OperandExpected(e.to_string()));
        }
    }

    while stack.len() > 0 {
        ans.push(stack.pop().unwrap());
    }

    Ok(ans)
}

fn rev_polish_op(elem: &ArithElem,
                 stack: &mut Vec<ArithElem>, ans: &mut Vec<ArithElem>) -> bool {
    loop {
        match stack.last() {
            None => {
                stack.push(elem.clone());
                break;
            },
            Some(_) => {
                let last = stack.last().unwrap();
                if last.order() < elem.order() 
                || (last.order() == 2 && elem.order() == 2) { // assignment
                    stack.push(elem.clone());
                    break;
                }
                ans.push(stack.pop().unwrap());
            },
        }
    }
    true
}
