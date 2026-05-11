//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod substr;

use core::fmt;
use core::fmt::Debug;

impl Clone for Box<dyn BracedParamExtension> {
    fn clone(&self) -> Box<dyn BracedParamExtension> {
        self.boxed_clone()
    }
}

impl Debug for dyn BracedParamExtension {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}

pub trait BracedParamExtension {
    fn boxed_clone(&self) -> Box<dyn BracedParamExtension>;
    fn get_text(&self) -> String;
}
