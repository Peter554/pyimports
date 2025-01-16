use crate::package_info::{Module, Package, PackageInfo, PackageItem, PackageItemToken};
use crate::prelude::*;
use crate::pypath::Pypath;
use anyhow::Result;
use itertools::Itertools;
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub(crate) fn build_package_info(data: &HashMap<Pypath, HashSet<Pypath>>) -> Result<PackageInfo> {
    let all_pypaths = data.keys().cloned();

    let mut items: SlotMap<PackageItemToken, PackageItem> = SlotMap::with_key();
    let mut items_by_pypath = HashMap::new();

    // `sorted_by_key` here puts deepest pypaths first.
    for pypath in all_pypaths.sorted_by_key(|pypath| -count_dots(pypath)) {
        if items_by_pypath.contains_key(&pypath) {
            continue;
        }

        // foo.bar.baz => [foo.bar.baz, foo.bar, foo]
        let mut pypaths = {
            let mut pypaths = vec![pypath.clone()];
            while let Some(parent_pypath) = pypaths.last().unwrap().parent() {
                pypaths.push(parent_pypath);
            }
            pypaths
        };

        let mut parent: Option<PackageItemToken> = None;
        while let Some(pypath) = pypaths.pop() {
            if pypaths.is_empty() {
                let token = items.insert_with_key(|token| {
                    Module {
                        path: PathBuf::new(),
                        pypath: pypath.clone(),
                        is_init: false,
                        token,
                        parent: parent.unwrap(),
                    }
                    .into()
                });
                items_by_pypath.insert(pypath, token);
            } else if let Some(token) = items_by_pypath.get(&pypath) {
                parent = Some(*token)
            } else {
                let token = items.insert_with_key(|token| {
                    Package {
                        path: PathBuf::new(),
                        pypath: pypath.clone(),
                        token,
                        parent,
                        packages: HashSet::new(),
                        modules: HashSet::new(),
                        init_module: None,
                    }
                    .into()
                });
                items_by_pypath.insert(pypath, token);
                parent = Some(token)
            }
        }
    }

    // Add init modules.
    for package in items.clone().values().filter_packages() {
        let pypath: Pypath = (package.pypath.to_string() + ".__init__").parse().unwrap();
        let token = items.insert_with_key(|token| {
            Module {
                path: PathBuf::new(),
                pypath: pypath.clone(),
                is_init: true,
                token,
                parent: package.token,
            }
            .into()
        });
        items_by_pypath.insert(pypath, token);
        items
            .get_mut(package.token)
            .unwrap()
            .unwrap_package_mut()
            .init_module = Some(token);
    }

    // Add package children.
    for package in items.clone().values().filter_packages() {
        if let Some(parent) = package.parent {
            items
                .get_mut(parent)
                .unwrap()
                .unwrap_package_mut()
                .packages
                .insert(package.token);
        }
    }
    for module in items.clone().values().filter_modules() {
        items
            .get_mut(module.parent)
            .unwrap()
            .unwrap_package_mut()
            .modules
            .insert(module.token);
    }

    // Get root.
    let root = items
        .values()
        .filter_packages()
        .filter_map(|p| match p.parent {
            Some(_) => None,
            None => Some(p.token),
        })
        .collect::<Vec<_>>();
    assert_eq!(root.len(), 1);
    let root = root[0];

    Ok(PackageInfo {
        root,
        items,
        items_by_path: HashMap::new(),
        items_by_pypath,
    })
}

fn count_dots(s: &str) -> isize {
    s.chars().filter(|c| *c == '.').count() as isize
}
