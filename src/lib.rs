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
//!     let expanded_src = Context::new()
//!         .include("platform.glsl", platform)
//!         .include("common.glsl",common)
//!         .expand(main).unwrap();
//! }
//! ```
#[macro_use]
extern crate lazy_static;
extern crate regex;
mod error;

pub use error::Error;
use regex::Regex;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::marker::PhantomData;

/// A Context stores data required to expand source string inputs
#[derive(Debug, Default)]
pub struct Context<'a> {
    included_files: BTreeMap<String, String>,
    phantom: PhantomData<&'a String>,
}

impl<'a> Context<'a> {
    /// Returns an empty Context
    pub fn new() -> Self {
        Context {
            ..Default::default()
        }
    }

    /// Associates an #include name with a GLSL source string
    pub fn include<S>(&mut self, name: S, src: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.included_files.insert(name.into(), src.into());
        self
    }

    /// Recursively expands the #include directives within the GLSL source string and
    /// returns the expanded source string
    pub fn expand<S>(&self, src: S) -> Result<String, Error>
    where
        S: Into<String>,
    {
        let mut expanded_src = Vec::new();
        self.expand_recursive(
            // data structures to return to the user
            &mut expanded_src,
            // data structures internal to the algorithm
            &src.into(),
            None,
            &mut Vec::new(),
            &mut BTreeSet::new(),
        ).map(move |_| expanded_src.join("\n"))
    }

    fn expand_recursive(
        &'a self,
        expanded_src: &mut Vec<String>,
        src: &'a str,
        in_file: Option<&'a str>,
        include_stack: &mut Vec<&'a str>,
        include_set: &mut BTreeSet<&'a str>,
    ) -> Result<(), Error> {
        lazy_static! {
            static ref INCLUDE_RE: Regex = Regex::new(
                r#"^\s*#\s*(pragma\s*)?include\s+[<"](?P<file>.*)[>"]"#
            ).expect("failed to compile INCLUDE_RE regex");
        }
        let mut need_line_directive = false;
        // Iterate through each line in the src input
        // - If the line matches our INCLUDE_RE regex, recurse
        // - Otherwise, add the line to our outputs and continue to the next line
        for (line_num, line) in src.lines().enumerate() {
            if let Some(caps) = INCLUDE_RE.captures(line) {
                // The following expect should be impossible, but write a nice message anyways
                let cap_match = caps
                    .name("file")
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
                if let Some(src) = self.included_files.get(included_file) {
                    include_stack.push(&included_file);
                    self.expand_recursive(
                        expanded_src,
                        &src,
                        Some(included_file),
                        include_stack,
                        include_set,
                    )?;
                    include_stack.pop();
                    need_line_directive = true;
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
                if need_line_directive {
                    // add a #line directive to reset the line number so that GL compilation error
                    // messages contain line numbers that map to the users file
                    expanded_src.push(format!("#line {} 0", line_num + 1));
                }
                need_line_directive = false;
                expanded_src.push(String::from(line));
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
