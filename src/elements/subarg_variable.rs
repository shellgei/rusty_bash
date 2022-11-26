//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
//use crate::feeder::scanner::*;

use crate::abst_elems::ArgElem;

pub struct SubArgVariable {
    pub text: String,
    pub name: String,
    pub empty_option: String,
    pub empty_option_string: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgVariable {
    fn eval(&mut self, conf: &mut ShellCore, _as_value: bool) -> Vec<Vec<String>> {
        let val = conf.get_var(&self.name);

        if val.len() == 0 {
            vec!(vec!(self.empty_treat(conf)))
        }else if self.empty_option == ":+" {
            vec!(vec!(self.empty_option_string.clone()))
        }else{
            vec!(vec!(val))
        }
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgVariable {
    pub fn new(text: &mut Feeder) -> SubArgVariable {
        SubArgVariable {
            name: String::new(),
            text: String::new(),
            empty_option: String::new(),
            empty_option_string: String::new(),
            pos: DebugInfo::init(text),
        }
    }

    fn empty_treat(&self, conf: &mut ShellCore) -> String {
        let opt: &str = &self.empty_option.clone();

        match opt {
            "" => "".to_string(),
            ":-" => self.empty_option_string.clone(),
            ":=" => {
                conf.set_var(&self.name, &self.empty_option_string);
                self.empty_option_string.clone()
            },
            ":?" => {
                eprintln!("bash: {}: {}",self.name.clone(), self.empty_option_string.clone());
                conf.set_var("?", "1");
                "".to_string()
            },
            _ => "".to_string(),
        }
    }

    pub fn parse(text: &mut Feeder) -> Option<SubArgVariable> {
        if text.starts_with("${") {
            return SubArgVariable::parse_in_brace(text);
        }
        if ! text.starts_with("$") {
            return None;
        };

        let mut ans = SubArgVariable::new(text);
        ans.text = text.consume(1);
    
        let pos = text.scanner_name_or_parameter();
        ans.name = text.consume(pos);
        ans.text += &ans.name.clone();
        Some(ans)
    }
    
    fn parse_in_brace(text: &mut Feeder) -> Option<SubArgVariable> {
        let mut ans = SubArgVariable::new(text);
        let backup = text.clone();

        ans.text = text.consume(2);
        
        let pos = text.scanner_name_or_parameter();
        ans.name = text.consume(pos);
        ans.text += &ans.name.clone();

        if text.starts_with( ":-") || text.starts_with( ":=") 
            || text.starts_with( ":?") || text.starts_with( ":+") {
            ans.empty_option = text.consume(2);
            ans.text += &ans.empty_option.clone();

            let pos = text.scanner_until_escape("}");
            ans.empty_option_string = text.consume(pos);
            ans.text += &ans.empty_option_string.clone();
        }

        if text.starts_with("}") {
            ans.text += &text.consume(1);

            Some(ans)
        }else{
            text.rewind(backup);
            None
        }
    }
}
