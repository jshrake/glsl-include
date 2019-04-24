use regex::bytes::Regex as BytesRegex;

// Iterate over all the valid include directives
pub struct Directives<'a> {
    chars: &'a [u8],
    cursor: usize,
    line: usize,
    col: usize,
    ended: bool,
}

impl<'a> Directives<'a> {
    pub fn new(chars: &'a str) -> Self {
        Self {
            chars: chars.as_bytes(),
            cursor: 0,
            line: 0,
            col: 0,
            ended: false,
        }
    }

    // returns a slice with the remainder of the current line
    // advances cursor until the next line
    fn read_line(&mut self) -> Option<&'a [u8]> {
        if self.ended { return None }
        let b = self.cursor;
        loop {
            match self.next_byte() {
                (Some(b'\r'), Some(b'\n')) => {
                    let _ = self.next_byte();
                    break
                },
                (Some(b'\n'), _) => break,
                (None, _) => {
                    self.ended = true;
                    break
                },
                _ => {},
            }
        }
        if self.ended {
            Some(&self.chars[b..])
        } else {
            let e = self.cursor-1;
            Some(&self.chars[b..e])
        }
    }

    // consume until end of block comment
    // places the cursor at the character immediately after the end of the comment
    fn skip_block(&mut self) {
        loop {
            match self.next_byte() {
                (Some(b'*'), Some(b'/')) => {
                    let _ = self.next_byte();
                    return
                },
                (None, _) => return,
                _ => {},
            }
        }
    }

    fn next_byte(&mut self) -> (Option<u8>, Option<u8>) {
        let c0 = self._next();
        (c0, self._peek())
    }

    fn _peek(&mut self) -> Option<u8> {
        if self.ended { return None }
        self.chars.get(self.cursor).map(|c| *c)
    }

    fn _next(&mut self) -> Option<u8> {
        let n = self._peek();
        if n.is_some() { self.cursor += 1 }
        if n.map(|b| b == b'\n').unwrap_or(false) {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        n
    }
}

#[derive(Debug)]
pub struct Directive<'a> {
    pub included_file: &'a str,
    pub line_num: usize,
    // where the include directive starts
    pub start: usize,
    // where it ends
    pub end: usize,
    // where the current line ends.
    // this field is needed in order to pass the original tests.
    pub line_end: usize,
}

impl<'a> Iterator for Directives<'a> {
    type Item = Directive<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.next_byte() {
                (None, _) => return None,
                (Some(b'/'), Some(b'*')) => {
                    let _ = self.next_byte();
                    self.skip_block();
                },
                (Some(b'/'), Some(b'/')) => {
                    let _ = self.read_line();
                },
                (Some(b'#'), _) => {
                    // parse include directive using a Regex
                    lazy_static! {
                        static ref INCLUDE_RE: BytesRegex = BytesRegex::new(
                            r#"^\s*#\s*(pragma\s*)?include\s+[<"](?P<file>.*)[>"]\s*"#
                        ).expect("failed to compile INCLUDE_RE regex");
                    }

                    let cursor = self.cursor;
                    let line = self.line;
                    let col = self.col;

                    self.cursor -= 1;
                    self.col -= 1;
                    let line_rem = self.read_line().expect("this should always be Some");

                    if let Some(r) = INCLUDE_RE.find(line_rem) {
                        self.cursor = cursor + r.end() - 1;
                        self.line = line;
                        self.col = r.end();
                        // TODO don't panic when non-utf8
                        let path = INCLUDE_RE.captures(line_rem)
                            .and_then(|cap| cap.name("file"))
                            .and_then(|c| std::str::from_utf8(c.as_bytes()).ok())
                            .unwrap();

                        //return Some(Directive { path, line, start: col-1, end: col + r.end() - 1 })
                        return Some(Directive {
                            included_file: path,
                            line_num: line,
                            start: cursor - 1,
                            end: self.cursor,
                            line_end: cursor + line_rem.len() - 1,
                        })
                    } else {
                        // reset cursor right after the #
                        self.cursor = cursor;
                        self.line = line;
                        self.col = col;
                        self.ended = false;
                    }
                }
                _ => {},
            }
        }
    }
}

