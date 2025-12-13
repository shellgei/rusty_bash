//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{Data, DataBase};

impl DataBase {
    pub fn print(&mut self, name: &str) {
        if let Some(d) = self.get_ref(name) {
            Self::print_with_name(d, name, false);
        } else if let Some(f) = self.functions.get(name) {
            println!("{}", &f.text);
        }
    }

    pub fn print_for_declare(&mut self, name: &str) {
        if let Some(d) = self.get_ref(name) {
            Self::print_with_name(d, name, true);
        } else if let Some(f) = self.functions.get(name) {
            println!("{}", &f.text);
        }
    }

    fn print_with_name(d: &mut Box::<dyn Data>, name: &str, declare_print: bool) {
        let body = d.get_fmt_string();
        if !d.is_initialized() {
            println!("{name}");
        } else if declare_print
            && (d.is_single() || d.is_special() )
            && !body.starts_with("\"")
            && !body.ends_with("\"")
        {
            println!("{name}=\"{body}\"");
        } else {
            println!("{name}={body}");
        }
    }
}
