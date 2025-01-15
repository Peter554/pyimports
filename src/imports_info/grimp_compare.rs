use crate::imports_info::{ImportMetadata, ImportsInfo};
use crate::package_info::{PackageInfo, PackageItemIterator};
use crate::pypath::Pypath;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub(crate) fn build_imports_info(
    package_info: PackageInfo,
    data: &HashMap<Pypath, HashSet<Pypath>>,
) -> Result<ImportsInfo> {
    let mut imports_info = ImportsInfo {
        package_info: Arc::new(package_info.clone()),
        internal_imports: HashMap::new(),
        reverse_internal_imports: HashMap::new(),
        internal_imports_metadata: HashMap::new(),
        external_imports: HashMap::new(),
        external_imports_metadata: HashMap::new(),
    };

    imports_info.initialise_maps()?;

    // By definition, packages import their init modules.
    for package in package_info.get_all_items().filter_packages() {
        if let Some(init_module) = package.init_module() {
            imports_info.add_internal_import(
                package.token().into(),
                init_module.into(),
                ImportMetadata::ImplicitImport,
            )?;
        }
    }

    for (from_pypath, to_pypaths) in data {
        let mut from_item = package_info.get_item_by_pypath(from_pypath).unwrap();
        if from_item.is_package() {
            let init_module = from_item.unwrap_package().init_module().unwrap();
            from_item = package_info.get_item(init_module.into())?;
        }
        for to_pypath in to_pypaths {
            let to_item = package_info.get_item_by_pypath(to_pypath).unwrap();
            imports_info.add_internal_import(
                from_item.token(),
                to_item.token(),
                ImportMetadata::ExplicitImport {
                    line_number: 1,
                    is_typechecking: false,
                },
            )?;
        }
    }

    Ok(imports_info)
}
