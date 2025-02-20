use std::array::TryFromSliceError;
use std::io;
use std::num::TryFromIntError;
use std::sync::mpsc;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not value to type: {0}")]
    Cast(#[from] TryFromIntError),
    #[error("Could not get lines to display")]
    GetLines,
    #[error("Could not hilight search term")]
    Hilight(#[from] TryFromSliceError),
    #[error("Could not initialize terminal")]
    Io(#[from] io::Error),
    #[error("Could not parse regular expression")]
    RegexBuild(#[from] regex::Error),
    #[error("Error with search term")]
    SearchTerm(#[from] aho_corasick::BuildError),
    #[error("Could not read input")]
    StreamingReceive(#[from] mpsc::RecvError),
    #[error("Could not send input to terminal")]
    StreamingSend,
    #[error("Timeout while waiting for input stream")]
    StreamingTimeout(#[from] std::sync::mpsc::RecvTimeoutError),
}

impl From<Error> for io::Error {
    fn from(value: Error) -> Self {
        io::Error::other(value.to_string())
    }
}
