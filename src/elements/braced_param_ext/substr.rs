//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::{Word, WordMode};
use crate::error::parse::ParseError;
use super::BracedParamExtension;
use crate::elements::parameter::Parameter;
use crate::error::exec::ExecError;

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub text: String,
    pub offset: Word, //本来は計算式を入れられる
    pub length: Option<Word>, //同上
}

fn word_to_num(w: &Word, text: &str, core: &mut ShellCore)
-> Result<i32, ExecError> {
    let n = match w.eval_as_value(core)?.trim() {
        "" => 0,   //空文字や空白文字だけの場合は0扱い
        s => match s.parse::<i32>() { //数字に変換
            Ok(num) => num,
            Err(e) => return Err(ExecError::Other(e.to_string())),
        },
    };

    if n < 0 {  //マイナス指定のとき
        Ok(n + text.chars().count() as i32) //加工前の文字列の文字数を足す
    }else {
        Ok(n)
    }
}

impl BracedParamExtension for Substr {
    fn exec(&mut self, _: &Parameter, text: &str,
            core: &mut ShellCore) -> Result<String, ExecError> {
        if self.offset.text.is_empty() && self.length.is_none() {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }    //↑ echo ${A:}のようなパターンでエラーを返す
     
        let offset = word_to_num(&self.offset, text, core)?;
        if offset < 0 { //文字列の長さよりマイナスが大きければ空文字に
            return Ok("".to_string());
        }

        let ans = text.chars().enumerate()       //textを1文字ずつにバラして番号づけ
            .filter(|(i, _)| (*i as i32) >= offset)   //オフセット値より番号が大きい部分だけ残す
            .map(|(_, c)| c).collect::<String>(); //番号を除去してString型に戻す
        
        match self.length.as_ref() {
            None => Ok(ans),
            Some(w) => {
                let length = word_to_num(w, &ans, core)?;
                let ans = ans.chars()
                    .take(std::cmp::max(0, length) as usize)
                    .collect();

                Ok(ans)
            },
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> { Box::new(self.clone()) }
}

impl Substr {
    fn eat_length(&mut self, feeder: &mut Feeder,
                  core: &mut ShellCore) -> Result<(), ParseError> {
        if !feeder.starts_with(":") {
            return Ok(());
        }
        self.text += &feeder.consume(1);

        let mode = WordMode::PermitAnyUntil(vec!["}".to_string()]);
        self.length = Some( Word::parse(feeder, core, Some(mode))?.unwrap_or(Word::default()) );
        self.text += &self.length.as_mut().unwrap().text.clone();

        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with(":") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(1);

        let mode = WordMode::PermitAnyUntil(vec![":".to_string(), "}".to_string()]);
        ans.offset = Word::parse(feeder, core, Some(mode))?.unwrap_or(Word::default());
        ans.text += &ans.offset.text.clone();
        ans.eat_length(feeder, core)?;

//        dbg!("{:?}", &ans);
        Ok(Some(ans))
    }
}
