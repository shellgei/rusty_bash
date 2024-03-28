//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::env;

impl ShellCore {
    pub fn get_param_ref(&mut self, key: &str) -> &str { //パラメータの参照を返す
        if  self.parameters.get(key) == None { //キーがない場合
            if let Ok(val) = env::var(key) {   //環境変数がないか探す
                self.set_param(key, &val);     //環境変数があったら内部のパラメータとして取り込む
            }
        }

        match self.parameters.get(key) { //改めてキーに対応する値を探す
            Some(val) => val,            //あれば文字列の参照を返す
            None      => "",             //なければ空文字を返す
        }
    }

    pub fn set_param(&mut self, key: &str, val: &str) { //値のセット
        self.parameters.insert(key.to_string(), val.to_string());
    }
}
