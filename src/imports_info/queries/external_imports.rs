use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};

use anyhow::Result;

use crate::{Error, ImportMetadata, ImportsInfo, IntoPypath, PackageItemToken, Pypath};

/// An object that allows querying external imports.
pub struct ExternalImportsQueries<'a> {
    pub(crate) imports_info: &'a ImportsInfo,
}

impl<'a> ExternalImportsQueries<'a> {
    /// Returns a map of all the direct imports.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a",
    ///     "a.py" => "from django.db import models"
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_pkg = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage")?.unwrap()
    ///     .token();
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.external_imports().get_direct_imports(),
    ///     hashmap! {
    ///         root_pkg => hashset!{},
    ///         root_init => hashset!{},
    ///         a => hashset!{"django.db.models".parse()?},
    ///     }
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_direct_imports(&self) -> HashMap<PackageItemToken, HashSet<Pypath>> {
        self.imports_info.external_imports.clone()
    }

    /// Returns true if a direct import exists.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a",
    ///     "a.py" => "from django.db import models"
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    ///
    /// assert!(
    ///     imports_info.external_imports().direct_import_exists(a, "django.db.models")?,
    /// );
    /// assert!(
    ///     !imports_info.external_imports().direct_import_exists(root_init, "django.db.models")?,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn direct_import_exists<T: IntoPypath>(
        &self,
        from: PackageItemToken,
        to: T,
    ) -> Result<bool> {
        let to = to.into_pypath()?;

        self.imports_info.package_info.get_item(from)?;

        Ok(self
            .imports_info
            .external_imports
            .get(&from)
            .unwrap()
            .contains(to.borrow()))
    }

    /// Returns the items directly imported by the passed package item.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a",
    ///     "a.py" => "from django.db import models; import pydantic.BaseModel as BM"
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.external_imports().get_items_directly_imported_by(a)?,
    ///     hashset!{
    ///         "django.db.models".parse()?,
    ///         "pydantic.BaseModel".parse()?,
    ///     }
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_items_directly_imported_by(
        &'a self,
        item: PackageItemToken,
    ) -> Result<HashSet<Pypath>> {
        self.imports_info.package_info.get_item(item)?;

        Ok(self
            .imports_info
            .external_imports
            .get(&item)
            .unwrap()
            .clone())
    }

    /// Returns the metadata associated with the passed import.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo,ImportMetadata,ExplicitImportMetadata};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a",
    ///     "a.py" => "from django.db import models"
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.external_imports().get_import_metadata(a, "django.db.models")?,
    ///     &ImportMetadata::ExplicitImport(ExplicitImportMetadata {
    ///         line_number: 1,
    ///         is_typechecking: false
    ///     })
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_import_metadata<T: IntoPypath>(
        &'a self,
        from: PackageItemToken,
        to: T,
    ) -> Result<&'a ImportMetadata> {
        let to = to.into_pypath()?;
        if self.direct_import_exists(from, to.borrow())? {
            Ok(self
                .imports_info
                .external_imports_metadata
                .get(&(from, to.borrow().clone()))
                .unwrap())
        } else {
            Err(Error::NoSuchImport)?
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use maplit::{hashmap, hashset};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{testpackage, testutils::TestPackage, ExplicitImportMetadata, PackageInfo};

    #[test]
    fn test_get_direct_imports() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "import pydantic",
            "a.py" => "from django import db"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package = imports_info._item("testpackage");
        let root_package_init = imports_info._item("testpackage.__init__");
        let a = imports_info._item("testpackage.a");

        assert_eq!(
            imports_info.external_imports().get_direct_imports(),
            hashmap! {
                root_package => hashset!{},
                root_package_init => hashset! {"pydantic".parse()?},
                a => hashset! {"django.db".parse()?},
            }
        );

        Ok(())
    }

    #[test]
    fn test_get_items_directly_imported_by() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "import pydantic",
            "a.py" => "from django import db"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package_init = imports_info._item("testpackage.__init__");

        assert_eq!(
            imports_info
                .external_imports()
                .get_items_directly_imported_by(root_package_init)?,
            hashset! {"pydantic".parse()?}
        );

        Ok(())
    }

    #[test]
    fn test_get_import_metadata() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "import pydantic"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package_init = imports_info._item("testpackage.__init__");

        let external_imports = imports_info.external_imports();
        let metadata = external_imports.get_import_metadata(root_package_init, "pydantic")?;

        assert_eq!(
            metadata,
            &ImportMetadata::ExplicitImport(ExplicitImportMetadata {
                line_number: 1,
                is_typechecking: false
            })
        );

        Ok(())
    }
}
