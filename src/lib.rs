// Copyright 2018 glsl-include Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[macro_use]
extern crate lazy_static;
extern crate regex;
pub mod error;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use regex::Regex;
use error::Error;

#[derive(Debug, Default)]
pub struct Context<'a> {
    files: BTreeMap<&'a str, &'a str>,
    should_generate_source_map: bool,
}

/// A map from the index to FileLine items. The index corresponds to the
pub type SourceMap<'a> = Vec<FileLine<'a>>;

///
#[derive(Debug)]
pub struct FileLine<'a> {
    pub file: Option<&'a str>,
    pub line: usize,
}

impl<'a> Context<'a> {
    /// Returns a new Context
    pub fn new() -> Context<'a> {
        Context {
            ..Default::default()
        }
    }

    /// Associates an #include <name> with a source string
    pub fn include(mut self, name: &'a str, src: &'a str) -> Self {
        self.files.insert(name, src);
        self
    }

    /// Enables generating the source_map
    pub fn generate_source_map(mut self) -> Self {
        self.should_generate_source_map = true;
        self
    }

    pub fn expand(&self, src: &'a str) -> Result<(String, Option<SourceMap<'a>>), Error> {
        self.expand_raw(src)
            .map(|(result, source_map)| (result.join("\n"), source_map))
    }

    pub fn expand_raw(&self, src: &'a str) -> Result<(Vec<&'a str>, Option<SourceMap<'a>>), Error> {
        let mut result = Vec::new();
        let mut source_map = Vec::new();
        self.expand_recursive(
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

    fn expand_recursive(
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
                    let in_file = name.map(|s| s.to_string());
                    let problem_include = file.to_string();
                    let include_stack = include_stack.into_iter().map(|s| s.to_string()).collect();
                    return Err(Error::RecursiveInclude {
                        in_file: in_file,
                        line_num: line_num,
                        problem_include: problem_include,
                        include_stack: include_stack,
                    });
                }
                include_stack.push(&file);

                // the src may include files that haven't been specified with Context::file,
                if let Some(content) = self.files.get(file) {
                    self.expand_recursive(
                        Some(file),
                        content,
                        result,
                        source_map,
                        include_stack,
                        include_set,
                    )?;
                } else {
                    let in_file = name.map(|s| s.to_string());
                    let problem_include = file.to_string();
                    return Err(Error::FileNotFound {
                        in_file: in_file,
                        line_num: line_num,
                        problem_include: problem_include,
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
