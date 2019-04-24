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
mod iter;

pub use crate::error::Error;
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
    pub fn expand<S: AsRef<str>>(&self, src: S) -> Result<String, Error> {
        let mut expanded = String::new();
        self.expand_recursive(
            // data structures to return to the user
            &mut expanded,
            // data structures internal to the algorithm
            src.as_ref(),
            None,
            &mut Vec::new(),
            &mut BTreeSet::new(),
        )?;

        if expanded.is_empty() {
            // TODO Cow::Borrowed
            Ok(String::from(src.as_ref()))
        } else {
            // TODO Cow::Owned
            Ok(expanded)
        }
    }

    fn expand_recursive(
        &'a self,
        expanded: &mut String,
        src: &'a str,
        in_file: Option<&'a str>,
        include_stack: &mut Vec<&'a str>,
        include_set: &mut BTreeSet<&'a str>,
    ) -> Result<(), Error> {
        let includes: Vec<_> = iter::Directives::new(src).collect();

        if in_file.is_some() || !includes.is_empty() {
            let mut last = 0;

            for iter::Directive { included_file, line_num, start, end, line_end } in includes {
                // return if the included file already exists in the include_stack
                // this signals that we're in an infinite loop
                if include_stack.contains(&included_file) {
                    let in_file = in_file.map(|s| s.to_string());
                    let problem_include = included_file.to_string();
                    let include_stack = include_stack.into_iter().map(|s| s.to_string()).collect();
                    return Err(Error::RecursiveInclude {
                        in_file,
                        line_num,
                        problem_include,
                        include_stack,
                    });
                }

                // if the included file exists in our context, recurse
                if let Some(inlined) = self.included_files.get(included_file) {
                    expanded.push_str(&src[last..start]);

                    include_stack.push(&included_file);
                    self.expand_recursive(
                        expanded,
                        &inlined,
                        Some(included_file),
                        include_stack,
                        include_set,
                    )?;
                    include_stack.pop();

                    // TODO refactor
                    // :-( HACK :-( to pass the original tests
                    if line_end == end {
                        expanded.push_str(&format!("\n#line {} 0", line_num+2));
                    } else {
                        expanded.push_str(&format!("\n#line {} 0\n", line_num+1));
                        expanded.extend((start..end).map(|_| ' '));
                    }
                    last = end;
                } else {
                    let in_file = in_file.map(|s| s.to_string());
                    let problem_include = included_file.to_string();
                    return Err(Error::FileNotFound {
                        in_file,
                        line_num,
                        problem_include,
                    });
                }
            }

            expanded.push_str(&src[last..]);
        }

        Ok(())
    }
}
