//! Errors

use std::path::PathBuf;

use crate::package_info::{ModuleToken, PackageToken};
use crate::pypath::Pypath;
use rustpython_parser::ParseError;
use thiserror::Error;

#[allow(missing_docs)]
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("unknown package {0:?}")]
    UnknownPackage(PackageToken),

    #[error("unknown module {0:?}")]
    UnknownModule(ModuleToken),

    #[error("unable to parse python file {path}")]
    UnableToParsePythonFile {
        path: PathBuf,
        #[source]
        parse_error: ParseError,
    },

    #[error("unknown internal import {0}")]
    UnknownInternalImport(Pypath),

    #[error("no such import")]
    NoSuchImport,

    #[error("not a package")]
    NotAPackage,

    #[error("not a module")]
    NotAModule,

    #[error("invalid pypath")]
    InvalidPypath,
}
