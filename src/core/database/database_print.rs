//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{Data, DataBase};

impl DataBase {
    pub fn print_params_and_funcs(&mut self) {
        self.get_param_keys()
            .into_iter()
            .for_each(|k| self.print_param(&k));
        self.get_func_keys()
            .into_iter()
            .for_each(|k| { self.print_func(&k); });
    }

    pub fn print_param(&mut self, name: &str) {
        if let Some(d) = self.get_ref(name) {
            Self::print_with_name(d, name, false);
        }
    }

    pub fn print_func(&mut self, name: &str) -> bool {
        if let Some(f) = self.functions.get_mut(name) {
            f.pretty_print(0);
            return true;
        }
        false
    }

    pub fn print_for_declare(&mut self, name: &str) {
        if let Some(d) = self.get_ref(name) {
            Self::print_with_name(d, name, true);
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
