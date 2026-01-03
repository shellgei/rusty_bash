//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod pipe;
pub mod redirect;

use crate::elements::io::redirect::Redirect;
use crate::elements::Pipe;
use crate::error::exec::ExecError;
use crate::ShellCore;

pub fn connect(
    pipe: &mut Pipe,
    rs: &mut [Redirect],
    core: &mut ShellCore,
) -> Result<(), ExecError> {
    pipe.connect(core)?;

    for r in rs.iter_mut() {
        r.connect(false, core)?;
    }
    Ok(())
}
