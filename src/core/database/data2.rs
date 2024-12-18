//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod array;
pub mod assoc;
pub mod single;
pub mod special;

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
    fn set_as_single(&mut self, _: &str) -> bool {false}
    fn get_as_single(&mut self) -> Option<String> {None}
    fn set_as_assoc(&mut self, _: &str, _: &str) -> bool {false}
    fn get_as_assoc(&mut self, _: &str) -> Option<String> {None}
    fn set_as_array(&mut self, _: &str, _: &str) -> bool {false}
    fn get_as_array(&mut self, _: &str) -> Option<String> {None}
    fn get_all_as_array(&mut self) -> Option<Vec<String>> {None}
    fn is_special(&self) -> bool {false}
    fn is_single(&self) -> bool {false}
    fn is_assoc(&self) -> bool {false}
    fn is_array(&self) -> bool {false}
    fn len(&mut self) -> usize;
}