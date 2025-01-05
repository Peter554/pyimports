use std::borrow::Borrow;
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use derive_more::derive::{Display, Into};
use derive_more::Deref;
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display, Into)]
pub struct Pypath(String);

impl Pypath {
    pub(crate) fn new(s: &str) -> Pypath {
        Pypath(s.to_string())
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

    /// Returns true if this pypath is equal to or an ancestor of the passed pypath.
    ///
    /// # Example
    ///
    /// ```
    /// use pyimports::Pypath;
    ///
    /// let foo_bar: Pypath = "foo.bar".parse().unwrap();
    /// let foo_bar_baz: Pypath = "foo.bar.baz".parse().unwrap();
    ///
    /// assert!(foo_bar.is_equal_to_or_ancestor_of(&foo_bar_baz));
    /// assert!(!foo_bar_baz.is_equal_to_or_ancestor_of(&foo_bar));
    /// ```
    pub fn is_equal_to_or_ancestor_of(&self, other: &Pypath) -> bool {
        self == other || other.0.starts_with(&(self.0.clone() + "."))
    }

    /// Returns true if this pypath is equal to or a descendant of the passed pypath.
    ///
    /// # Example
    ///
    /// ```
    /// use pyimports::Pypath;
    ///
    /// let foo_bar: Pypath = "foo.bar".parse().unwrap();
    /// let foo_bar_baz: Pypath = "foo.bar.baz".parse().unwrap();
    ///
    /// assert!(!foo_bar.is_equal_to_or_descendant_of(&foo_bar_baz));
    /// assert!(foo_bar_baz.is_equal_to_or_descendant_of(&foo_bar));
    /// ```
    pub fn is_equal_to_or_descendant_of(&self, other: &Pypath) -> bool {
        other.is_equal_to_or_ancestor_of(self)
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
        let mut v = self.0.split(".").collect::<Vec<_>>();
        v.pop();
        Pypath(v.join("."))
    }
}

// TODO: Is there some way to achieve this via the TryFrom/TryInto trait?
/// A trait that can be used as a bound to generic functions that want
/// to accept a [`Pypath`], `&Pypath` or a `&str`.
///
/// ```
/// # use anyhow::Result;
/// use std::borrow::Borrow;
///
/// use pyimports::prelude::*;
/// use pyimports::Pypath;
///
/// fn f<T: IntoPypath>(pypath: T) -> Result<()> {
///     let pypath = pypath.into_pypath()?;
///     let pypath: &Pypath = pypath.borrow();
///     print!("{}", pypath);
///     Ok(())
/// }
///
/// # fn main() -> Result<()> {
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
    /// Convert into a [Pypath].
    fn into_pypath(self) -> Result<impl Borrow<Pypath>>;
}

impl<B: Borrow<Pypath>> IntoPypath for B {
    fn into_pypath(self) -> Result<impl Borrow<Pypath>> {
        Ok(self)
    }
}

impl IntoPypath for &str {
    fn into_pypath(self) -> Result<impl Borrow<Pypath>> {
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
    fn test_is_equal_or_ancestor() -> Result<()> {
        assert!(Pypath::new("foo.bar").is_equal_to_or_ancestor_of(&Pypath::new("foo.bar")));
        assert!(Pypath::new("foo.bar").is_equal_to_or_ancestor_of(&Pypath::new("foo.bar.baz")));
        assert!(!Pypath::new("foo.bar").is_equal_to_or_ancestor_of(&Pypath::new("foo")));

        Ok(())
    }

    #[test]
    fn test_is_equal_or_descendant() -> Result<()> {
        assert!(Pypath::new("foo.bar").is_equal_to_or_descendant_of(&Pypath::new("foo.bar")));
        assert!(!Pypath::new("foo.bar").is_equal_to_or_descendant_of(&Pypath::new("foo.bar.baz")));
        assert!(Pypath::new("foo.bar").is_equal_to_or_descendant_of(&Pypath::new("foo")));

        Ok(())
    }

    #[test]
    fn test_parent() -> Result<()> {
        assert_eq!(Pypath::new("foo.bar.baz").parent(), Pypath::new("foo.bar"));
        assert_eq!(Pypath::new("foo.bar").parent(), Pypath::new("foo"));

        Ok(())
    }
}
