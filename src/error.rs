use std;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    RecursiveInclude {
        in_file: Option<String>,
        line_num: usize,
        problem_include: String,
        include_stack: Vec<String>,
    },
    FileNotFound {
        in_file: Option<String>,
        line_num: usize,
        problem_include: String,
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
                ref in_file,
                ref problem_include,
                ref line_num,
                ref include_stack,
                ..
            } => write!(
                f,
                "Detected recursive include of file \"{}\" in file {:?}, line {}, include stack {:?}",
                problem_include, in_file, line_num, include_stack
            ),
            &Error::FileNotFound {
                ref problem_include,
                ref in_file,
                ref line_num,
                ..
            } => write!(
                f,
                "Could not find file \"{}\", included from file {:?}, line {}\nhelp: Call Context::file with the file name and contents",
                problem_include, in_file, line_num
            ),
        }
    }
}
