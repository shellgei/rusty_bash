//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{BracedParam, Variable};
use crate::elements::braced_param_ext;
use crate::elements::substitution::subscript::Subscript;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

impl BracedParam {
    fn eat_subscript(
        feeder: &mut Feeder,
        ans: &mut Self,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        if let Some(s) = Subscript::parse(feeder, core)? {
            ans.text += &s.text;
            if s.text.contains('@') && !ans.num {
                ans.treat_as_array = true;
            }
            ans.param.index = Some(s);
            return Ok(true);
        }

        Ok(false)
    }

    fn eat_param(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        if len != 0 {
            ans.param = Variable::default();
            ans.param.name = feeder.consume(len);
            ans.text += &ans.param.name;
            return true;
        }

        let mut len = feeder.scanner_uint(core);
        if len == 0 {
            len = feeder.scanner_special_and_positional_param();
        }

        if len != 0 {
            ans.param = Variable::default();
            ans.param.name = feeder.consume(len);
            ans.treat_as_array = ans.param.name == "@" && !ans.num;
            ans.text += &ans.param.name;
            return true;
        }

        feeder.starts_with("}")
    }

    fn eat_unknown(
        &mut self,
        feeder: &mut Feeder,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        if feeder.is_empty() {
            feeder.feed_additional_line(core)?;
        }

        if feeder.starts_with("}") {
            return Ok(true);
        }

        let len = match feeder.starts_with("\\}") || feeder.starts_with("\\\\") {
            true => 2,
            false => feeder.scanner_char(),
        };

        let unknown = feeder.consume(len);
        self.unknown += &unknown.clone();
        self.text += &unknown;
        Ok(false)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with("${") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(2);

        if feeder.starts_with("#") && !feeder.starts_with("#}") {
            ans.num = true;
            ans.text += &feeder.consume(1);
        } else if feeder.starts_with("!") {
            ans.indirect = true;
            ans.text += &feeder.consume(1);
        }

        if Self::eat_param(feeder, &mut ans, core) {
            Self::eat_subscript(feeder, &mut ans, core)?;

            if let Some(op) = braced_param_ext::parse(feeder, core)? {
                ans.text += &op.get_text();
                ans.extension = Some(op);
            }
        }
        while !ans.eat_unknown(feeder, core)? {}

        ans.text += &feeder.consume(1);
        Ok(Some(ans))
    }
}
