//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use super::elem::ArithElem;

pub fn rearrange(elements: &[ArithElem]) -> Result<Vec<ArithElem>, ExecError> {
    let mut ans = vec![];
    let mut stack: Vec<ArithElem> = vec![];

    for e in elements {
        match e.is_operand() {
            true  => ans.push(e.clone()),
            false => rev_polish_op(&e, &mut stack, &mut ans),
        };
    }

    while stack.len() > 0 {
        ans.push(stack.pop().unwrap());
    }

    Ok(ans)
}

fn rev_polish_op(elem: &ArithElem, stack: &mut Vec<ArithElem>,
                 ans: &mut Vec<ArithElem>) {
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
}
