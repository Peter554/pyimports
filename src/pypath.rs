use std::borrow::Borrow;
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

/// The absolute dotted path to a python package/module/member.
///
/// # Example
///
/// ```
/// use pyimports::Pypath;
///
/// let result  = "foo.bar".parse::<Pypath>();
/// assert!(result.is_ok());
///
/// // Relative paths are not allowed.
/// let result  = ".foo.bar".parse::<Pypath>();
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pypath {
    pub(crate) s: String,
}

impl Pypath {
    pub(crate) fn new(s: &str) -> Pypath {
        Pypath { s: s.to_string() }
    }
}

impl fmt::Display for Pypath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.s)
    }
}

impl FromStr for Pypath {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if PYPATH_REGEX.is_match(s) {
            Ok(Pypath::new(s))
        } else {
            Err(Error::InvalidPypath)
        }
    }
}

impl AsRef<str> for Pypath {
    fn as_ref(&self) -> &str {
        &self.s
    }
}

impl From<Pypath> for String {
    fn from(value: Pypath) -> Self {
        value.s
    }
}

impl<'a> From<&'a Pypath> for &'a str {
    fn from(value: &'a Pypath) -> Self {
        &value.s
    }
}

impl Pypath {
    pub(crate) fn from_path(path: &Path, root_path: &Path) -> Result<Self> {
        let path = path.strip_prefix(root_path.parent().unwrap())?;
        let mut s = path.to_str().unwrap();
        if s.ends_with(".py") {
            s = s.strip_suffix(".py").unwrap();
        }
        let s = s.replace("/", ".");
        Ok(Pypath::new(&s))
    }

    /// Returns true if the passed pypath is contained by this pypath.
    ///
    /// # Example
    ///
    /// ```
    /// use pyimports::Pypath;
    ///
    /// let foo_bar: Pypath = "foo.bar".parse().unwrap();
    /// let foo_bar_baz: Pypath = "foo.bar.baz".parse().unwrap();
    ///
    /// assert!(foo_bar.contains(&foo_bar_baz));
    /// assert!(!foo_bar_baz.contains(&foo_bar));
    /// ```
    pub fn contains(&self, other: &Pypath) -> bool {
        self == other || other.s.starts_with(&(self.s.clone() + "."))
    }

    /// Returns true if this pypath is contained by the passed pypath.
    ///
    /// # Example
    ///
    /// ```
    /// use pyimports::Pypath;
    ///
    /// let foo_bar: Pypath = "foo.bar".parse().unwrap();
    /// let foo_bar_baz: Pypath = "foo.bar.baz".parse().unwrap();
    ///
    /// assert!(!foo_bar.is_contained_by(&foo_bar_baz));
    /// assert!(foo_bar_baz.is_contained_by(&foo_bar));
    /// ```
    pub fn is_contained_by(&self, other: &Pypath) -> bool {
        other.contains(self)
    }

    /// Returns the parent of this pypath.
    ///
    /// # Example
    ///
    /// ```
    /// use pyimports::Pypath;
    ///
    /// let foo_bar: Pypath = "foo.bar".parse().unwrap();
    /// let foo_bar_baz: Pypath = "foo.bar.baz".parse().unwrap();
    ///
    ///assert!(foo_bar_baz.parent() == foo_bar);
    /// ```
    pub fn parent(&self) -> Self {
        let mut v = self.s.split(".").collect::<Vec<_>>();
        v.pop();
        Pypath { s: v.join(".") }
    }
}

// TODO: Is there some way to achieve this via the TryFrom/TryInto trait?
/// A trait that can be used as a bound to generic functions that want
/// to accept a [`Pypath`], `&Pypath` or a `&str`.
///
/// ```
/// use std::borrow::Borrow;
///
/// use anyhow::Result;
///
/// use pyimports::{IntoPypath,Pypath};
///
/// fn f<T: IntoPypath>(pypath: T) -> Result<()> {
///     let pypath = pypath.into_pypath()?;
///     let pypath: &Pypath = pypath.borrow();
///     print!("{}", pypath);
///     Ok(())
/// }
///
/// # fn main() -> Result<()> {
///
/// // `f` can accept a Pypath
/// f("foo.bar".parse::<Pypath>()?)?;
/// // ...or a &Pypath
/// f(&"foo.bar".parse::<Pypath>()?)?;
/// // ...or a &str
/// f("foo.bar")?;
/// # Ok(())
/// # }
/// ```
pub trait IntoPypath {
    ///
    fn into_pypath(&self) -> Result<impl Borrow<Pypath>>;
}

impl<B: Borrow<Pypath>> IntoPypath for B {
    fn into_pypath(&self) -> Result<impl Borrow<Pypath>> {
        Ok(self.borrow())
    }
}

impl IntoPypath for &str {
    fn into_pypath(&self) -> Result<impl Borrow<Pypath>> {
        let pypath = self.parse::<Pypath>()?;
        Ok(pypath)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_pypath_from_str() -> Result<()> {
        assert!(Pypath::from_str("foo").is_ok());
        assert!(Pypath::from_str("foo.bar").is_ok());

        assert!(matches!(
            Pypath::from_str(".foo.bar"),
            Err(Error::InvalidPypath)
        ));
        assert!(matches!(
            Pypath::from_str("foo.bar."),
            Err(Error::InvalidPypath)
        ));

        Ok(())
    }

    #[test]
    fn test_contains() -> Result<()> {
        assert!(Pypath::new("foo.bar").contains(&Pypath::new("foo.bar")));
        assert!(Pypath::new("foo.bar").contains(&Pypath::new("foo.bar.baz")));
        assert!(!Pypath::new("foo.bar").contains(&Pypath::new("foo")));

        Ok(())
    }

    #[test]
    fn test_contained_by() -> Result<()> {
        assert!(Pypath::new("foo.bar").is_contained_by(&Pypath::new("foo.bar")));
        assert!(!Pypath::new("foo.bar").is_contained_by(&Pypath::new("foo.bar.baz")));
        assert!(Pypath::new("foo.bar").is_contained_by(&Pypath::new("foo")));

        Ok(())
    }

    #[test]
    fn test_parent() -> Result<()> {
        assert_eq!(Pypath::new("foo.bar.baz").parent(), Pypath::new("foo.bar"));
        assert_eq!(Pypath::new("foo.bar").parent(), Pypath::new("foo"));

        Ok(())
    }
}
