// Copyright 2018 glsl_include Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

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
        let result = self.run_impl(None, src, Vec::new(), &mut Vec::new());
        Ok(result.join("\n"))
    }

    fn run_impl(
        &self,
        name: Option<&'a str>,
        src: &'a str,
        mut result: Vec<&'a str>,
        include_stack: &mut Vec<&'a str>,
    ) -> Vec<&'a str> {
        lazy_static! {
            static ref INCLUDE_RE : Regex = Regex::new(r#"^\s*#\s*include\s+[<"](?P<file>.*)[>"]"#).unwrap();
        }

        // iterate through each line in src, if it matches INCLUDE_RE, then recurse, otherwise,
        // push the line to the result buffer and continue
        for line in src.lines() {
            if let Some(caps) = INCLUDE_RE.captures(line) {
                // I'm not sure how this could ever panic
                let cap_match = caps.name("file")
                    .expect("Could not find capture group with name \"file\"");
                let file = cap_match.as_str();

                // if this file is already in our include stack, panic
                if include_stack.contains(&file) {
                    let name = name.unwrap();
                    panic!(
                        "Detected recursive file include in \"{}\" @ line \"{}\". include stack: {:?}",
                        name, line, include_stack
                    );
                }
                include_stack.push(&file);

                // the src may include files that haven't been specified with Preprocessor::file,
                match self.files.get(file) {
                    Some(content) => {
                        let mut content_result =
                            self.run_impl(Some(file), content, Vec::new(), include_stack);
                        result.append(&mut content_result);
                    }
                    None => {
                        panic!(
                        "Could not find file \"{file}\". help: need to call Preprocessor::file(\"{file}\", \"content\")",
                        file = file);
                    }
                };

                include_stack.pop();
            } else {
                result.push(line);
            }
        }
        result
    }
}
