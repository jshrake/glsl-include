#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::collections::BTreeMap;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
}

#[derive(Debug)]
pub struct Preprocessor<'a> {
    files: BTreeMap<&'a str, &'a str>,
}

pub type SourceMap = Vec<(Option<String>, usize)>;

impl<'a> Preprocessor<'a> {
    pub fn new() -> Preprocessor<'a> {
        Preprocessor {
            files: BTreeMap::new(),
        }
    }

    pub fn file(mut self, path: &'a str, src: &'a str) -> Self {
        self.files.insert(path, src);
        self
    }

    pub fn run(&self, src: &'a str) -> Result<String, Error> {
        let result = self.run_entry(src, Vec::new());
        Ok(result.join("\n"))
    }

    fn run_entry(&self, src: &'a str, mut result: Vec<&'a str>) -> Vec<&'a str> {
        lazy_static! {
            static ref INCLUDE_RE : Regex = Regex::new(r#"^\s*#\s*include\s+[<"](?P<file>.*)[>"]"#).unwrap();
        }
        for line in src.lines() {
            if let Some(caps) = INCLUDE_RE.captures(line) {
                let file = &caps["file"];
                if let Some(content) = self.files.get(file) {
                    let mut content_result = self.run_entry(content, Vec::new());
                    result.append(&mut content_result);
                } else {
                    panic!("{} not found", file);
                }
            } else {
                result.push(line);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0() {
        let src = "
#version 410
#include <hi.glsl>
#include <bye.glsl>
#if ok
#include \"ok.glsl\"
#else
#include \"bad.glsl\"
#endif
void main() {
}
        ";
        println!("{}", src);
        let mut p = Preprocessor::new();
        p = p.file("hi.glsl", "#include <aloha.glsl>\nvoid hi () {}");
        p = p.file("bye.glsl", "void bye () {}");
        p = p.file("ok.glsl", "void ok () {}");
        p = p.file("bad.glsl", "void bad () {}");
        p = p.file("aloha.glsl", "void aloha () {}");
        let result = p.run(src).unwrap();
        println!("{}", result);
    }
}
