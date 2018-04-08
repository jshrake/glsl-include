// Copyright 2018 glsl-include Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    RecursiveInclude(String, String, Vec<String>),
    FileNotFound(Option<String>, String),
}

#[derive(Debug)]
pub struct Preprocessor<'a> {
    files: BTreeMap<&'a str, &'a str>,
}

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
        self.run_raw(src).map(|result| result.join("\n"))
    }

    pub fn run_raw(&self, src: &'a str) -> Result<Vec<&'a str>, Error> {
        self.run_impl(None, src, Vec::new(), &mut Vec::new(), &mut BTreeSet::new())
    }

    fn run_impl(
        &self,
        name: Option<&'a str>,
        src: &'a str,
        mut result: Vec<&'a str>,
        include_stack: &mut Vec<&'a str>,
        include_set: &mut BTreeSet<&'a str>,
    ) -> Result<Vec<&'a str>, Error> {
        lazy_static! {
            static ref INCLUDE_RE : Regex = Regex::new(r#"^\s*#\s*include\s+[<"](?P<file>.*)[>"]"#).expect("failed to compile INCLUDE_RE regex");
        }

        // iterate through each line in src, if it matches INCLUDE_RE, then recurse, otherwise,
        // push the line to the result buffer and continue
        for line in src.lines() {
            if let Some(caps) = INCLUDE_RE.captures(line) {
                // The following expect should be impossible, but write a nice message anyways
                let cap_match = caps.name("file")
                    .expect("Could not find capture group with name \"file\"");
                let file = cap_match.as_str();

                // if this file has already been included, continue to the next line
                if include_set.contains(&file) {
                    continue;
                }

                // if this file is already in our include stack, return Err
                if include_stack.contains(&file) {
                    let name = name.expect("impossible").to_string();
                    let line = line.to_string();
                    let stack = include_stack.into_iter().map(|s| s.to_string()).collect();
                    return Err(Error::RecursiveInclude(name, line, stack));
                }
                include_stack.push(&file);

                // the src may include files that haven't been specified with Preprocessor::file,
                match self.files.get(file) {
                    Some(content) => {
                        let mut content_result = self.run_impl(
                            Some(file),
                            content,
                            Vec::new(),
                            include_stack,
                            include_set,
                        )?;
                        result.append(&mut content_result);
                    }
                    None => {
                        let name = name.map(|s| s.to_string());
                        let file = file.to_string();
                        return Err(Error::FileNotFound(name, file));
                    }
                };
                include_stack.pop();
            } else {
                result.push(line);
            }
        }
        // We're done processing this file, if it has a name, add it to the include_set
        if let Some(name) = name {
            include_set.insert(name);
        }
        Ok(result)
    }
}
