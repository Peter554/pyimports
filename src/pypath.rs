use std::fmt;
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

use crate::Error;

lazy_static! {
    static ref ABSOLUTE_PYPATH_REGEX: Regex = Regex::new(r"^\w+(\.\w+)*$").unwrap();
}

/// A dotted path to a python module/module-member.
/// An absolute path (not a relative path).
///
/// # Example
///
/// ```
/// use pyimports::AbsolutePyPath;
///
/// let result  = "foo.bar".parse::<AbsolutePyPath>();
/// assert!(result.is_ok());
///
/// // Relative paths are not allowed.
/// let result  = ".foo.bar".parse::<AbsolutePyPath>();
/// assert!(result.is_err());
/// ```
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
        if ABSOLUTE_PYPATH_REGEX.is_match(s) {
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

impl<'a> From<&'a AbsolutePyPath> for &'a str {
    fn from(value: &'a AbsolutePyPath) -> Self {
        &value.s
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

    /// Returns true if the passed pypath is contained by this pypath.
    ///
    /// # Example
    ///
    /// ```
    /// use pyimports::AbsolutePyPath;
    ///
    /// let foo_bar: AbsolutePyPath = "foo.bar".parse().unwrap();
    /// let foo_bar_baz: AbsolutePyPath = "foo.bar.baz".parse().unwrap();
    ///
    /// assert!(foo_bar.contains(&foo_bar_baz));
    /// assert!(!foo_bar_baz.contains(&foo_bar));
    /// ```
    pub fn contains(&self, other: &AbsolutePyPath) -> bool {
        self == other || other.s.starts_with(&(self.s.clone() + "."))
    }

    /// Returns true if this pypath is contained by the passed pypath.
    ///
    /// # Example
    ///
    /// ```
    /// use pyimports::AbsolutePyPath;
    ///
    /// let foo_bar: AbsolutePyPath = "foo.bar".parse().unwrap();
    /// let foo_bar_baz: AbsolutePyPath = "foo.bar.baz".parse().unwrap();
    ///
    /// assert!(!foo_bar.is_contained_by(&foo_bar_baz));
    /// assert!(foo_bar_baz.is_contained_by(&foo_bar));
    /// ```
    pub fn is_contained_by(&self, other: &AbsolutePyPath) -> bool {
        other.contains(self)
    }

    /// Returns the parent of this pypath.
    ///
    /// # Example
    ///
    /// ```
    /// use pyimports::AbsolutePyPath;
    ///
    /// let foo_bar: AbsolutePyPath = "foo.bar".parse().unwrap();
    /// let foo_bar_baz: AbsolutePyPath = "foo.bar.baz".parse().unwrap();
    ///
    ///assert!(foo_bar_baz.parent() == foo_bar);
    /// ```
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
            AbsolutePyPath::from_str(".foo.bar"),
            Err(Error::InvalidPyPath)
        ));
        assert!(matches!(
            AbsolutePyPath::from_str("foo.bar."),
            Err(Error::InvalidPyPath)
        ));

        Ok(())
    }

    #[test]
    fn test_contains() -> Result<()> {
        assert!(AbsolutePyPath::new("foo.bar").contains(&AbsolutePyPath::new("foo.bar")));
        assert!(AbsolutePyPath::new("foo.bar").contains(&AbsolutePyPath::new("foo.bar.baz")));
        assert!(!AbsolutePyPath::new("foo.bar").contains(&AbsolutePyPath::new("foo")));

        Ok(())
    }

    #[test]
    fn test_contained_by() -> Result<()> {
        assert!(AbsolutePyPath::new("foo.bar").is_contained_by(&AbsolutePyPath::new("foo.bar")));
        assert!(
            !AbsolutePyPath::new("foo.bar").is_contained_by(&AbsolutePyPath::new("foo.bar.baz"))
        );
        assert!(AbsolutePyPath::new("foo.bar").is_contained_by(&AbsolutePyPath::new("foo")));

        Ok(())
    }

    #[test]
    fn test_parent() -> Result<()> {
        assert_eq!(
            AbsolutePyPath::new("foo.bar.baz").parent(),
            AbsolutePyPath::new("foo.bar")
        );
        assert_eq!(
            AbsolutePyPath::new("foo.bar").parent(),
            AbsolutePyPath::new("foo")
        );

        Ok(())
    }
}
