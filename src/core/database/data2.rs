//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod assoc;

use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Data2 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.print_body()).finish()
    }
}

impl Clone for Box::<dyn Data2> {
    fn clone(&self) -> Box<dyn Data2> {
        self.boxed_clone()
    }
}

pub trait Data2 {
    fn boxed_clone(&self) -> Box<dyn Data2>;
    fn print_body(&self) -> String;
    fn set_as_assoc(&mut self, _: &str, _: &str) -> bool {false}
    fn get_as_assoc(&mut self, _: &str) -> Option<String> {None}
    fn is_assoc(&self) -> bool {false}
}
