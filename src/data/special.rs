//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::data::Data;

#[derive(Debug, Clone)]
pub struct SpecialData {
    pub data: String,
    pub dynamic_get: fn(&mut Data) -> String,
}
