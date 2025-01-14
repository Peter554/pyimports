use crate::imports_info::ImportsInfo;
use crate::package_info::grimp_compare::build_package_info;
use crate::pypath::Pypath;
use anyhow::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub fn build_imports_info<T: AsRef<Path>>(path: T) -> Result<ImportsInfo> {
    let data = read_data_file(path.as_ref())?;
    let package_info = build_package_info(&data)?;
    crate::imports_info::grimp_compare::build_imports_info(package_info, &data)
}

fn read_data_file<T: AsRef<Path>>(path: T) -> Result<HashMap<Pypath, HashSet<Pypath>>> {
    let s = fs::read_to_string(path.as_ref())?;

    let v: Value = serde_json::from_str(&s)?;
    let v = v.as_object().unwrap();

    let v = v
        .into_iter()
        .map(|(k, v)| {
            (
                k.parse().unwrap(),
                v.as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().parse().unwrap())
                    .collect(),
            )
        })
        .collect();

    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::{hashmap, hashset};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_build_imports_info() -> Result<()> {
        let imports_info = build_imports_info("./data/small_graph.json")?;

        assert_eq!(
            imports_info
                .package_info()
                .get_all_items()
                .map(|item| item.to_string())
                .collect::<HashSet<_>>(),
            hashset! {
                "Package(pkg)".into(),
                "Module(pkg.__init__)".into(),
                "Package(pkg.animals)".into(),
                "Module(pkg.animals.__init__)".into(),
                "Module(pkg.animals.dog)".into(),
                "Module(pkg.animals.cat)".into(),
                "Package(pkg.food)".into(),
                "Module(pkg.food.__init__)".into(),
                "Module(pkg.food.meat)".into(),
                "Module(pkg.food.fish)".into(),
            }
        );

        let root_pkg = imports_info.package_info()._item("pkg");
        let root_init = imports_info.package_info()._item("pkg.__init__");
        let animals_pkg = imports_info.package_info()._item("pkg.animals");
        let animals_init = imports_info.package_info()._item("pkg.animals.__init__");
        let dog = imports_info.package_info()._item("pkg.animals.dog");
        let cat = imports_info.package_info()._item("pkg.animals.cat");
        let food_pkg = imports_info.package_info()._item("pkg.food");
        let food_init = imports_info.package_info()._item("pkg.food.__init__");
        let meat = imports_info.package_info()._item("pkg.food.meat");
        let fish = imports_info.package_info()._item("pkg.food.fish");

        assert_eq!(
            imports_info.internal_imports().get_direct_imports(),
            hashmap! {
                root_pkg => hashset! {root_init},
                root_init => hashset! {},
                animals_pkg => hashset! {animals_init},
                animals_init => hashset! {},
                dog => hashset! {meat, fish},
                cat => hashset! {fish},
                food_pkg => hashset! {food_init},
                food_init => hashset! {},
                meat => hashset! {},
                fish => hashset! {}
            }
        );

        let _ = build_imports_info("./data/large_graph.json")?;

        Ok(())
    }   
}
