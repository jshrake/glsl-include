// Copyright 2018 glsl-include Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! # glsl-include
//!
//! glsl-include is a library for expanding #include directives in GLSL source
//! strings
//!
//! ```rust
//! extern crate glsl_include;
//! use glsl_include::Context;
//!
//! fn main () {
//!     let main = r"
//!         #version 410
//!         #include <platform.glsl>
//!         #include <common.glsl>
//!         out vec4 fragColor;
//!         void main () {
//!             fragColor = vec4(1.0);
//!         }";
//!     let platform = "void platform_fn() {}";
//!     let common = "uniform float iTime;";
//!     let (expanded_src, source_map) = Context::new()
//!         .include("platform.glsl", platform)
//!         .include("common.glsl",common)
//!         .expand_to_string(main).unwrap();
//! }
//! ```
#[macro_use]
extern crate lazy_static;
extern crate regex;
pub mod error;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use regex::Regex;
use error::Error;

/// A Context stores data required to expand source string inputs
#[derive(Debug, Default)]
pub struct Context<'a> {
    included_files: BTreeMap<&'a str, &'a str>,
}

/// A map from the expanded source line number to the corresponding include file line number
pub type SourceMap<'a> = Vec<FileLine<'a>>;

/// An include file, line number pair
///
/// A value of None for `file` corresponds to a line in the source string provided to [method@expand]
#[derive(Debug)]
pub struct FileLine<'a> {
    pub file: Option<&'a str>,
    pub line: usize,
}

impl<'a> Context<'a> {
    /// Returns an empty Context
    pub fn new() -> Context<'a> {
        Context {
            ..Default::default()
        }
    }

    /// Associates an #include name with a GLSL source string
    pub fn include(mut self, name: &'a str, src: &'a str) -> Self {
        self.included_files.insert(name, src);
        self
    }

    /// Recursively expands the #include directives within the GLSL source string and
    /// returns the expanded source and source map
    pub fn expand(&self, src: &'a str) -> Result<(Vec<&'a str>, SourceMap<'a>), Error> {
        let mut expanded_src = Vec::new();
        let mut source_map = Vec::new();
        self.expand_recursive(
            None,
            src,
            // data structures to return to the user
            &mut expanded_src,
            &mut source_map,
            // data structures internal to the algorithm
            &mut Vec::new(),
            &mut BTreeSet::new(),
        ).map(move |_| (expanded_src, source_map))
    }

    /// Like [`expand`](#method.expand) but joins the expanded source with newlines
    pub fn expand_to_string(&self, src: &'a str) -> Result<(String, SourceMap<'a>), Error> {
        self.expand(src)
            .map(|(expanded_src, source_map)| (expanded_src.join("\n"), source_map))
    }

    /// Like [`expand`](#method.expand) but maps the expanded source to a Vec of &[u8]
    pub fn expand_to_bytes(&self, src: &'a str) -> Result<(Vec<&[u8]>, SourceMap<'a>), Error> {
        self.expand(src).map(|(expanded_src, source_map)| {
            (
                expanded_src.into_iter().map(|x| x.as_bytes()).collect(),
                source_map,
            )
        })
    }

    fn expand_recursive(
        &self,
        in_file: Option<&'a str>,
        src: &'a str,
        expanded_src: &mut Vec<&'a str>,
        source_map: &mut SourceMap<'a>,
        include_stack: &mut Vec<&'a str>,
        include_set: &mut BTreeSet<&'a str>,
    ) -> Result<(), Error> {
        lazy_static! {
            static ref INCLUDE_RE : Regex = Regex::new(r#"^\s*#\s*include\s+[<"](?P<file>.*)[>"]"#).expect("failed to compile INCLUDE_RE regex");
        }

        // Iterate through each line in the src input
        // - If the line matches our INCLUDE_RE regex, recurse
        // - Otherwise, add the line to our outputs and continue to the next line
        for (line_num, line) in src.lines().enumerate() {
            if let Some(caps) = INCLUDE_RE.captures(line) {
                // The following expect should be impossible, but write a nice message anyways
                let cap_match = caps.name("file")
                    .expect("Could not find capture group with name \"file\"");
                let included_file = cap_match.as_str();

                // if this file has already been included, continue to the next line
                // this acts as a header guard
                if include_set.contains(&included_file) {
                    continue;
                }

                // return if the included file already exists in the include_stack
                // this signals that we're in an infinite loop
                if include_stack.contains(&included_file) {
                    let in_file = in_file.map(|s| s.to_string());
                    let problem_include = included_file.to_string();
                    let include_stack = include_stack.into_iter().map(|s| s.to_string()).collect();
                    return Err(Error::RecursiveInclude {
                        in_file: in_file,
                        line_num: line_num,
                        problem_include: problem_include,
                        include_stack: include_stack,
                    });
                }

                // if the included file exists in our context, recurse
                if let Some(content) = self.included_files.get(included_file) {
                    include_stack.push(&included_file);
                    self.expand_recursive(
                        Some(included_file),
                        content,
                        expanded_src,
                        source_map,
                        include_stack,
                        include_set,
                    )?;
                    include_stack.pop();
                } else {
                    let in_file = in_file.map(|s| s.to_string());
                    let problem_include = included_file.to_string();
                    return Err(Error::FileNotFound {
                        in_file: in_file,
                        line_num: line_num,
                        problem_include: problem_include,
                    });
                }
            } else {
                // Got a regular line
                expanded_src.push(line);
                source_map.push(FileLine {
                    file: in_file,
                    line: line_num,
                });
            }
        }

        // Add the in_file to the include set to prevent
        // future inclusions
        if let Some(in_file) = in_file {
            include_set.insert(in_file);
        }
        Ok(())
    }
}
