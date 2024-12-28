use std::fmt;
use std::path::Path;

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyPath {
    s: String,
}

impl From<&str> for PyPath {
    fn from(value: &str) -> Self {
        PyPath {
            s: value.to_string(),
        }
    }
}

impl From<String> for PyPath {
    fn from(value: String) -> Self {
        PyPath { s: value }
    }
}

impl From<PyPath> for String {
    fn from(value: PyPath) -> Self {
        value.s
    }
}

impl fmt::Display for PyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.s)
    }
}

impl PyPath {
    pub(crate) fn from_path(path: &Path, root_path: &Path) -> Result<Self> {
        let path = path.strip_prefix(root_path.parent().unwrap())?;
        let mut s = path.to_str().unwrap();
        if s.ends_with(".py") {
            s = s.strip_suffix(".py").unwrap();
        }
        Ok(s.replace("/", ".").into())
    }

    pub fn is_relative(&self) -> bool {
        self.s.starts_with(".")
    }

    pub fn resolve_relative(&self, path: &Path, root_path: &Path) -> Self {
        if !self.is_relative() {
            panic!()
        }
        let trimmed_pypath = self.s.trim_start_matches(".");
        let base_pypath = {
            let n = self.s.len() - trimmed_pypath.len();
            let mut base_path = path;
            for _ in 0..n {
                base_path = base_path.parent().unwrap();
            }
            PyPath::from_path(base_path, root_path).unwrap()
        };
        PyPath {
            s: base_pypath.s + "." + trimmed_pypath,
        }
    }

    pub fn contains(&self, other: &PyPath) -> bool {
        if self.is_relative() || other.is_relative() {
            panic!()
        }
        self == other || other.s.starts_with(&(self.s.clone() + "."))
    }

    pub fn is_contained_by(&self, other: &PyPath) -> bool {
        other.contains(self)
    }

    pub fn parent(&self) -> Self {
        let mut v = self.s.split(".").collect::<Vec<_>>();
        v.pop();
        PyPath { s: v.join(".") }
    }
}
