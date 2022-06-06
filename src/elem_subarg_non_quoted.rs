//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
use crate::scanner::*;

use crate::abst_elem_argelem::ArgElem;

pub struct SubArgNonQuoted {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgNonQuoted {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String> {
        vec!(self.text.clone())
    }
}

impl SubArgNonQuoted {
    pub fn parse(text: &mut Feeder) -> Option<SubArgNonQuoted> {
        let pos = scanner_until_escape(text, 0, " \n\t\"';{}()$<>&");
        if pos == 0{
            return None;
        };
        Some( SubArgNonQuoted{text: text.consume(pos), pos: DebugInfo::init(text) } )
    }

    pub fn parse2(text: &mut Feeder) -> Option<SubArgNonQuoted> {
        let pos = scanner_until_escape(text, 0, " \n\t\"';)$<>&");
        if pos == 0{
            return None;
        };
        Some( SubArgNonQuoted{text: text.consume(pos), pos: DebugInfo::init(text) } )
    }

    pub fn parse3(text: &mut Feeder) -> Option<SubArgNonQuoted> {
        if text.match_at(0, ",}"){
            return None;
        };
        
        let pos = scanner_until_escape(text, 0, ",{}()");
        Some( SubArgNonQuoted{ text: text.consume(pos), pos: DebugInfo::init(text) })
    }

    pub fn parse4(text: &mut Feeder) -> Option<SubArgNonQuoted> {
        if text.nth(0) == '"' {
            return None;
        };
    
        let pos = scanner_until_escape(text, 0, "\"$");
        Some( SubArgNonQuoted{text: text.consume(pos), pos: DebugInfo::init(text)})
    }
}

/*
pub struct SubArgDoubleQuoted {
    pub text: String,
    pub pos: DebugInfo,
    pub subargs: Vec<Box<dyn ArgElem>>
}

impl ArgElem for SubArgDoubleQuoted {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let mut text = "".to_string();
        for a in &mut self.subargs {
            let sub = a.eval(conf);
            text += &sub[0];
        };

        let s = text.replace("\\", "\\\\").replace("*", "\\*"); 
        vec!(s)
    }

    fn text(&self) -> String {
        self.text.clone()
    }

    /*
    fn get_length(&self) -> usize {
        self.text.len()
    }
    */
}


impl SubArgDoubleQuoted {
/* parser for a string such as "aaa${var}" */
    pub fn parse(text: &mut Feeder) -> Option<SubArgDoubleQuoted> {
        let backup = text.clone();
    
        let mut ans = SubArgDoubleQuoted {
            text: "".to_string(),
            pos: DebugInfo::init(text),
            subargs: vec!(),
        };
    
        if scanner_until(text, 0, "\"") != 0 {
            return None;
        }
        text.consume(1);
    
        loop {
            if let Some(a) = SubArgVariable::parse2(text) {
                ans.subargs.push(Box::new(a));
            }else if let Some(a) = SubArgCommandExp::parse(text) {
                ans.subargs.push(Box::new(a));
            }else if let Some(a) = SubArgVariable::parse(text) {
                ans.subargs.push(Box::new(a));
            }else if let Some(a) = SubArgNonQuoted::parse4(text) {
                ans.subargs.push(Box::new(a));
            }else{
                break;
            };
        }
    
        if scanner_until(text, 0, "\"") != 0 {
            text.rewind(backup);
            return None;
        }
        text.consume(1);
    
        let mut text = "\"".to_string();
        for a in &ans.subargs {
            text += &a.text();
        }
        ans.text = text + "\"";
    
        Some(ans)
    }
}

pub struct SubArgSingleQuoted {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgSingleQuoted {
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String> {
        let strip = self.text[1..self.text.len()-1].to_string();
        let s = strip.replace("\\", "\\\\").replace("*", "\\*"); 
        vec!(s)
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgSingleQuoted {
    pub fn parse(text: &mut Feeder) -> Option<SubArgSingleQuoted> {
        if !text.match_at(0, "'"){
            return None;
        };
    
        let pos = scanner_until(text, 1, "'");
        Some(SubArgSingleQuoted{text: text.consume(pos+1), pos: DebugInfo::init(text)})
    }
}

pub struct SubArgBraced {
    pub text: String,
    pub pos: DebugInfo,
    pub args: Vec<Arg>
}

impl ArgElem for SubArgBraced {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        if self.args.len() == 0{
            return vec!("{}".to_string());
        }else if self.args.len() == 1{
            let mut ans = "{".to_string();
            for s in self.args[0].eval(conf) {
                ans += &s;
            };
            ans += "}";
            return vec!(ans);
        };

        let mut ans = vec!();
        for arg in &mut self.args {
            ans.append(&mut arg.eval(conf));
        };
        ans
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgBraced {
    pub fn parse(text: &mut Feeder) -> Option<SubArgBraced> {
        let pos = scanner_until(text, 0, "{");
        if pos != 0 {
            return None;
        }
        
        let mut ans = SubArgBraced {
            text: text.consume(1),
            pos: DebugInfo::init(text),
            args: vec!(),
        };
    
        while let Some(arg) = arg_in_brace(text) {
            ans.text += &arg.text.clone();
            ans.args.push(arg); 
    
            if scanner_until(text, 0, ",") == 0 {
                ans.text += &text.consume(1);
                continue;
            }else if scanner_until(text, 0, "}") == 0 {
                ans.text += &text.consume(1);
                break;
            };
        };
    
        Some(ans)
    }
}

pub struct SubArgVariable {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgVariable {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let name = if self.text.rfind('}') == Some(self.text.len()-1) {
            self.text[2..self.text.len()-1].to_string()
        }else{
            self.text[1..].to_string()
        };
        vec!(conf.get_var(&name))
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgVariable {
    pub fn parse(text: &mut Feeder) -> Option<SubArgVariable> {
        if !(text.nth(0) == '$') || text.nth(1) == '{' {
            return None;
        };
    
        let pos = scanner_varname(&text, 1);
        Some(
            SubArgVariable{
                text: text.consume(pos),
                pos: DebugInfo::init(text),
            })
    }
    
    pub fn parse2(text: &mut Feeder) -> Option<SubArgVariable> {
        if !(text.nth(0) == '$' && text.nth(1) == '{') {
            return None;
        }
    
        let pos = scanner_varname(&text, 2);
        if text.nth(pos) == '}' {
            Some( SubArgVariable{ text: text.consume(pos+1), pos: DebugInfo::init(text) })
        }else{
            None
        }
    }
}

pub struct SubArgCommandExp {
    pub text: String,
    pub pos: DebugInfo,
    pub com: Command, 
}

impl ArgElem for SubArgCommandExp {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        self.com.expansion = true;
        vec!(self.com.exec(conf).replace("\n", " "))
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgCommandExp {
    pub fn parse(text: &mut Feeder) -> Option<SubArgCommandExp> {
        if !(text.nth(0) == '$' && text.nth(1) == '(') {
            return None;
        }
    
        let pos = scanner_end_of_bracket(text, 2, ')');
        let mut sub_feeder = Feeder::new_with(text.from_to(2, pos));
    
        if let Some(e) = Command::parse(&mut sub_feeder){
            let ans = Some (SubArgCommandExp {
                text: text.consume(pos+1),
                pos: DebugInfo::init(text),
                com: e }
            );
    
            return ans;
        };
        None
    }
}


*/
