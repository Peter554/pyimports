use std::path::PathBuf;

use rustpython_parser::ParseError;
use thiserror::Error;

use crate::{ModuleToken, PackageToken, Pypath};

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
