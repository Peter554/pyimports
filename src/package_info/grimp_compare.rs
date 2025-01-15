use crate::package_info::{Module, ModuleToken, Package, PackageInfo, PackageToken};
use crate::pypath::Pypath;
use anyhow::Result;
use itertools::Itertools;
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub(crate) fn build_package_info(data: &HashMap<Pypath, HashSet<Pypath>>) -> Result<PackageInfo> {
    let all_pypaths = data.keys().cloned();

    let mut packages: SlotMap<PackageToken, Package> = SlotMap::with_key();
    let mut modules: SlotMap<ModuleToken, Module> = SlotMap::with_key();
    let mut packages_by_pypath = HashMap::new();
    let mut modules_by_pypath = HashMap::new();

    // `sorted_by_key` here puts deepest pypaths first.
    for pypath in all_pypaths.sorted_by_key(|pypath| -count_dots(pypath)) {
        if packages_by_pypath.contains_key(&pypath) {
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

        let mut parent: Option<PackageToken> = None;
        while let Some(pypath) = pypaths.pop() {
            if pypaths.is_empty() {
                let token = modules.insert_with_key(|token| Module {
                    path: PathBuf::new(),
                    pypath: pypath.clone(),
                    is_init: false,
                    token,
                    parent: parent.unwrap(),
                });
                modules_by_pypath.insert(pypath, token);
            } else if let Some(token) = packages_by_pypath.get(&pypath) {
                parent = Some(*token)
            } else {
                let token = packages.insert_with_key(|token| Package {
                    path: PathBuf::new(),
                    pypath: pypath.clone(),
                    token,
                    parent,
                    packages: HashSet::new(),
                    modules: HashSet::new(),
                    init_module: None,
                });
                packages_by_pypath.insert(pypath, token);
                parent = Some(token)
            }
        }
    }

    // Add init modules.
    for package in packages.values_mut() {
        let pypath: Pypath = (package.pypath.to_string() + ".__init__").parse().unwrap();
        let token = modules.insert_with_key(|token| Module {
            path: PathBuf::new(),
            pypath: pypath.clone(),
            is_init: true,
            token,
            parent: package.token,
        });
        modules_by_pypath.insert(pypath, token);
        package.init_module = Some(token);
    }

    // Add package children.
    for package in packages.clone().values() {
        if let Some(parent) = package.parent {
            packages
                .get_mut(parent)
                .unwrap()
                .packages
                .insert(package.token);
        }
    }
    for module in modules.clone().values() {
        packages
            .get_mut(module.parent)
            .unwrap()
            .modules
            .insert(module.token);
    }

    // Get root.
    let root = packages
        .values()
        .filter_map(|p| match p.parent {
            Some(_) => None,
            None => Some(p.token),
        })
        .collect::<Vec<_>>();
    assert_eq!(root.len(), 1);
    let root = root[0];

    Ok(PackageInfo {
        root,
        packages,
        modules,
        packages_by_path: HashMap::new(),
        packages_by_pypath,
        modules_by_path: HashMap::new(),
        modules_by_pypath,
    })
}

fn count_dots(s: &str) -> isize {
    s.chars().filter(|c| *c == '.').count() as isize
}
