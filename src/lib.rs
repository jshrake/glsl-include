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
use std::fmt;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    RecursiveInclude {
        file: Option<String>,
        recursive_file: String,
        line_num: usize,
        include_stack: Vec<String>,
    },
    FileNotFound {
        file: Option<String>,
        not_found: String,
        line_num: usize,
    },
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::RecursiveInclude { .. } => "Detected recursive #include",
            &Error::FileNotFound { .. } => "Could not find #include file",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::RecursiveInclude {
                ref file,
                ref recursive_file,
                ref line_num,
                ref include_stack,
                ..
            } => write!(
                f,
                "Detected recursive include of file \"{}\" in file {:?}, line {}, include stack {:?}",
                recursive_file, file, line_num, include_stack
            ),
            &Error::FileNotFound {
                ref not_found,
                ref file,
                ref line_num,
                ..
            } => write!(
                f,
                "Could not find file \"{}\", included from file {:?}, line {}\nhelp: Call Preprocessor::file with the file name and contents",
                not_found, file, line_num
            ),
        }
    }
}

#[derive(Debug, Default)]
pub struct Preprocessor<'a> {
    files: BTreeMap<&'a str, &'a str>,
    should_generate_source_map: bool,
}

#[derive(Debug)]
pub struct FileLine<'a> {
    pub file: Option<&'a str>,
    pub line: usize,
}

pub type SourceMap<'a> = Vec<FileLine<'a>>;

impl<'a> Preprocessor<'a> {
    pub fn new() -> Preprocessor<'a> {
        Preprocessor {
            ..Default::default()
        }
    }

    pub fn file(mut self, path: &'a str, src: &'a str) -> Self {
        self.files.insert(path, src);
        self
    }

    pub fn generate_source_map(mut self) -> Self {
        self.should_generate_source_map = true;
        self
    }

    pub fn run(&self, src: &'a str) -> Result<(String, Option<SourceMap<'a>>), Error> {
        self.run_raw(src)
            .map(|(result, source_map)| (result.join("\n"), source_map))
    }

    pub fn run_raw(&self, src: &'a str) -> Result<(Vec<&'a str>, Option<SourceMap<'a>>), Error> {
        let mut result = Vec::new();
        let mut source_map = Vec::new();
        self.run_recursive(
            None,
            src,
            &mut result,
            &mut source_map,
            &mut Vec::new(),
            &mut BTreeSet::new(),
        ).map(move |_| {
            (
                result,
                if self.should_generate_source_map {
                    Some(source_map)
                } else {
                    None
                },
            )
        })
    }

    fn run_recursive(
        &self,
        name: Option<&'a str>,
        src: &'a str,
        result: &mut Vec<&'a str>,
        source_map: &mut SourceMap<'a>,
        include_stack: &mut Vec<&'a str>,
        include_set: &mut BTreeSet<&'a str>,
    ) -> Result<(), Error> {
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
                let line_num = result.len();

                // if this file is already in our include stack, return Err
                if include_stack.contains(&file) {
                    let stack = include_stack.into_iter().map(|s| s.to_string()).collect();
                    return Err(Error::RecursiveInclude {
                        file: name.map(|s| s.to_string()),
                        recursive_file: file.to_string(),
                        line_num: line_num,
                        include_stack: stack,
                    });
                }
                include_stack.push(&file);

                // the src may include files that haven't been specified with Preprocessor::file,
                if let Some(content) = self.files.get(file) {
                    self.run_recursive(
                        Some(file),
                        content,
                        result,
                        source_map,
                        include_stack,
                        include_set,
                    )?;
                } else {
                    return Err(Error::FileNotFound {
                        file: name.map(|s| s.to_string()),
                        not_found: file.to_string(),
                        line_num: line_num,
                    });
                }
                include_stack.pop();
            } else {
                result.push(line);
                if self.should_generate_source_map {
                    source_map.push(FileLine {
                        file: name,
                        line: result.len(),
                    });
                }
            }
        }
        // We're done processing this file, if it has a name, add it to the include_set
        if let Some(name) = name {
            include_set.insert(name);
        }
        Ok(())
    }
}
