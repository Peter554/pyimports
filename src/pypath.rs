use std::fmt;
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

use crate::Error;

lazy_static! {
    static ref PYPATH_REGEX: Regex = Regex::new(r"^\w+(\.\w+)*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsolutePyPath {
    pub(crate) s: String,
}

impl AbsolutePyPath {
    pub(crate) fn new(s: &str) -> AbsolutePyPath {
        AbsolutePyPath { s: s.to_string() }
    }
}

impl FromStr for AbsolutePyPath {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if PYPATH_REGEX.is_match(s) {
            Ok(AbsolutePyPath::new(s))
        } else {
            Err(Error::InvalidPyPath)
        }
    }
}

impl From<AbsolutePyPath> for String {
    fn from(value: AbsolutePyPath) -> Self {
        value.s
    }
}

impl fmt::Display for AbsolutePyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.s)
    }
}

impl AbsolutePyPath {
    pub(crate) fn from_path(path: &Path, root_path: &Path) -> Result<Self> {
        let path = path.strip_prefix(root_path.parent().unwrap())?;
        let mut s = path.to_str().unwrap();
        if s.ends_with(".py") {
            s = s.strip_suffix(".py").unwrap();
        }
        let s = s.replace("/", ".");
        Ok(AbsolutePyPath::new(&s))
    }

    pub fn contains(&self, other: &AbsolutePyPath) -> bool {
        self == other || other.s.starts_with(&(self.s.clone() + "."))
    }

    pub fn is_contained_by(&self, other: &AbsolutePyPath) -> bool {
        other.contains(self)
    }

    pub fn parent(&self) -> Self {
        let mut v = self.s.split(".").collect::<Vec<_>>();
        v.pop();
        AbsolutePyPath { s: v.join(".") }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_pypath_from_str() -> Result<()> {
        assert!(AbsolutePyPath::from_str("foo").is_ok());
        assert!(AbsolutePyPath::from_str("foo.bar").is_ok());

        assert!(matches!(
            AbsolutePyPath::from_str(".foo"),
            Err(Error::InvalidPyPath)
        ));

        Ok(())
    }
}
