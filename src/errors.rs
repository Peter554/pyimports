use std::path::{Path, PathBuf};

use rustpython_parser::ParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to parse python file: {path}")]
    UnableToParsePythonFile {
        path: PathBuf,
        #[source]
        parse_error: ParseError,
    },
}
