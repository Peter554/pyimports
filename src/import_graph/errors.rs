use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot read directory")]
    CannotReadDir(#[source] io::Error),

    #[error("not a python package")]
    NotAPythonPackage,

    #[error("package not found")]
    PackageNotFound(String),

    #[error("module not found")]
    ModuleNotFound(String),

    #[error("import not found")]
    ImportNotFound(String, String),
}
