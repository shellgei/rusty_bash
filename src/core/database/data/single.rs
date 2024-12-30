//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;

#[derive(Debug, Clone)]
pub struct SingleData {
    body: String,
}

impl From<&str> for SingleData {
    fn from(s: &str) -> Self {
        Self { body: s.to_string() }
    }
}

impl Data for SingleData {
    fn boxed_clone(&self) -> Box<dyn Data> { Box::new(self.clone()) }
    fn print_body(&self) -> String { self.body.clone() }

    fn set_as_single(&mut self, value: &str) -> Result<(), String> {
        self.body = value.to_string();
        Ok(())
    }

    fn get_as_single(&mut self) -> Option<String> { Some(self.body.clone()) }
    fn len(&mut self) -> usize { self.body.chars().count() }
    fn is_single(&self) -> bool {true}
}
