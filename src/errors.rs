//! Errors

use crate::package_info::PackageItemToken;
use crate::pypath::Pypath;
use rustpython_parser::ParseError;
use std::path::PathBuf;
use thiserror::Error;

#[allow(missing_docs)]
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("unknown package item {0:?}")]
    UnknownPackageItem(PackageItemToken),

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
