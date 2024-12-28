use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::{AbsolutePyPath, Error, ImportMetadata, ImportsInfo, PackageItemToken};

pub struct ExternalImportsQueries<'a> {
    pub(crate) imports_info: &'a ImportsInfo,
}

impl<'a> ExternalImportsQueries<'a> {
    pub fn get_direct_imports(&self) -> HashMap<PackageItemToken, HashSet<AbsolutePyPath>> {
        self.imports_info.external_imports.clone()
    }

    pub fn direct_import_exists(
        &self,
        from: PackageItemToken,
        to: &AbsolutePyPath,
    ) -> Result<bool> {
        self.imports_info.package_info.get_item(from)?;

        Ok(self
            .imports_info
            .external_imports
            .get(&from)
            .unwrap()
            .contains(to))
    }

    pub fn get_items_directly_imported_by(
        &'a self,
        item: PackageItemToken,
    ) -> Result<HashSet<AbsolutePyPath>> {
        self.imports_info.package_info.get_item(item)?;

        Ok(self
            .imports_info
            .external_imports
            .get(&item)
            .unwrap()
            .clone())
    }

    pub fn get_import_metadata(
        &'a self,
        from: PackageItemToken,
        to: &AbsolutePyPath,
    ) -> Result<Option<&'a ImportMetadata>> {
        if self.direct_import_exists(from, to)? {
            Ok(self
                .imports_info
                .external_imports_metadata
                .get(&(from, to.to_owned())))
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
    use crate::{testpackage, testutils::TestPackage, PackageInfo};

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
        let metadata =
            external_imports.get_import_metadata(root_package_init, &"pydantic".parse()?)?;

        assert_eq!(
            metadata,
            Some(&ImportMetadata {
                line_number: 1,
                is_typechecking: false
            })
        );

        Ok(())
    }
}
