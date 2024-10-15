//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;

pub fn eval(word: &mut Word, core: &mut ShellCore) -> bool {
    word.subwords.iter_mut().all(|sw| { sw.substitute(core) } )
}
