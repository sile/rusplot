#[macro_use]
extern crate trackable;

use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use trackable::error::{Failure, Failed};

pub type Result<T> = ::std::result::Result<T, Failure>;

macro_rules! track_io {
    ($expr:expr) => {
        track!($expr.map_err(Failure::from_error))
    }
}

#[derive(Debug)]
pub enum Input {
    Raw(String),
    File(FileInput),
}
impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Input::Raw(ref s) => write!(f, "{}", s),
            Input::File(ref i) => write!(f, "{}", i),
        }
    }
}
impl<'a> From<&'a str> for Input {
    fn from(f: &'a str) -> Self {
        Input::Raw(f.to_string())
    }
}
impl From<FileInput> for Input {
    fn from(f: FileInput) -> Self {
        Input::File(f)
    }
}

#[derive(Debug)]
pub struct FileInput {
    path: PathBuf,
    using: Option<String>, // TODO
    with: Option<Style>,
}
impl FileInput {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileInput {
            path: path.as_ref().to_path_buf(),
            using: None,
            with: None,
        }
    }
    pub fn using(mut self, using: &str) -> Self {
        self.using = Some(using.to_string());
        self
    }
    pub fn with(mut self, style: Style) -> Self {
        self.with = Some(style);
        self
    }
}
impl fmt::Display for FileInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.path)?;
        if let Some(ref s) = self.using {
            write!(f, " using {}", s)?;
        }
        if let Some(ref s) = self.with {
            write!(f, " with {}", s)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Style {
    Points,
    Lines,
    LinesPoints,
}
impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Style::Points => write!(f, "points"),
            Style::Lines => write!(f, "lines"),
            Style::LinesPoints => write!(f, "linespoints"),
        }
    }
}

#[derive(Debug)]
pub struct Plot {
    args: Vec<String>,
    inputs: Vec<Input>,

    // TODO: s/String/Range/
    range: Option<String>,
}
impl Plot {
    pub fn new() -> Self {
        Plot {
            args: Vec::new(),
            inputs: Vec::new(),
            range: None,
        }
    }
    pub fn persist(&mut self) -> &mut Self {
        self.args.push("--persist".to_string());
        self
    }
    pub fn input<T: Into<Input>>(&mut self, input: T) -> &mut Self {
        self.inputs.push(input.into());
        self
    }
    pub fn range(&mut self, range: &str) -> &mut Self {
        self.range = Some(range.to_string());
        self
    }
    pub fn show(&self) -> Result<Output> {
        let mut command = Command::new("gnuplot");
        command.args(self.args.clone());
        command.stdin(Stdio::piped());
        let mut child = track_io!(command.spawn())?;
        {
            let stdin = track_assert_some!(child.stdin.as_mut(), Failed);
            track_io!(write!(stdin, "plot "))?;
            if let Some(ref range) = self.range {
                track_io!(write!(stdin, "{} ", range))?;
            }
            track_io!(write!(
                stdin,
                "{} ",
                self.inputs
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ))?;
            track_io!(writeln!(stdin))?;
        }
        let output = track_io!(child.wait_with_output())?;
        Ok(output)
    }
}

#[cfg(test)]
mod test {
    #![allow(dead_code)]
    use super::*;

    #[test]
    fn it_works2() {
        let output = track_try_unwrap!(
            Plot::new()
                .persist()
                .input(FileInput::new("dataset/prices").using("1:2").with(
                    Style::Lines,
                ))
                .input(FileInput::new("dataset/prices").using("1:3").with(
                    Style::LinesPoints,
                ))
                .show()
        );
        println!("{:?}", output);
    }

    // #[test]
    fn it_works() {
        let output = track_try_unwrap!(
            Plot::new()
                .persist()
                .range("[][-2:2]")
                .input("sin(x)")
                .input("x")
                .input("x-(x**3)")
                .show()
        );
        println!("{:?}", output);
    }
}
