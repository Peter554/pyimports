use maplit::{hashmap, hashset};
use std::path::Path;

use pyimports::{Error, ImportGraphBuilder};

#[test]
fn test_packages() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph.packages(),
        hashset! {
            "somesillypackage",
            "somesillypackage.child1",
            "somesillypackage.child2",
            "somesillypackage.child3",
            "somesillypackage.child4",
            "somesillypackage.child5",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_modules() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph.modules(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.a",
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.d",
            "somesillypackage.e",
            "somesillypackage.z",
            //
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.a",
            "somesillypackage.child1.b",
            "somesillypackage.child1.c",
            "somesillypackage.child1.d",
            "somesillypackage.child1.e",
            "somesillypackage.child1.z",
            //
            "somesillypackage.child2.__init__",
            //
            "somesillypackage.child3.__init__",
            //
            "somesillypackage.child4.__init__",
            //
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_package_from_module() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .package_from_module("somesillypackage.__init__")
            .unwrap(),
        "somesillypackage"
    );
    assert_eq!(
        import_graph
            .package_from_module("somesillypackage.a")
            .unwrap(),
        "somesillypackage"
    );
    assert_eq!(
        import_graph
            .package_from_module("somesillypackage.child1.__init__")
            .unwrap(),
        "somesillypackage.child1"
    );
    assert_eq!(
        import_graph
            .package_from_module("somesillypackage.child1.a")
            .unwrap(),
        "somesillypackage.child1"
    );
}

#[test]
fn test_child_packages() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph.child_packages("somesillypackage").unwrap(),
        hashset! {
            "somesillypackage.child1",
            "somesillypackage.child2",
            "somesillypackage.child3",
            "somesillypackage.child4",
            "somesillypackage.child5",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_child_modules() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph.child_modules("somesillypackage").unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.a",
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.d",
            "somesillypackage.e",
            "somesillypackage.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .child_modules("somesillypackage.child1")
            .unwrap(),
        hashset! {
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.a",
            "somesillypackage.child1.b",
            "somesillypackage.child1.c",
            "somesillypackage.child1.d",
            "somesillypackage.child1.e",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .child_modules("somesillypackage.child2")
            .unwrap(),
        hashset! {
            "somesillypackage.child2.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .child_modules("somesillypackage.child3")
            .unwrap(),
        hashset! {
            "somesillypackage.child3.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .child_modules("somesillypackage.child4")
            .unwrap(),
        hashset! {
            "somesillypackage.child4.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .child_modules("somesillypackage.child5")
            .unwrap(),
        hashset! {
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_descendant_packages() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .descendant_packages("somesillypackage")
            .unwrap(),
        hashset! {
            "somesillypackage.child1",
            "somesillypackage.child2",
            "somesillypackage.child3",
            "somesillypackage.child4",
            "somesillypackage.child5",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .descendant_packages("somesillypackage.child1")
            .unwrap(),
        hashset! {}
    );
}

#[test]
fn test_descendant_modules() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph.descendant_modules("somesillypackage").unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.a",
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.d",
            "somesillypackage.e",
            "somesillypackage.z",
            //
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.a",
            "somesillypackage.child1.b",
            "somesillypackage.child1.c",
            "somesillypackage.child1.d",
            "somesillypackage.child1.e",
            "somesillypackage.child1.z",
            //
            "somesillypackage.child2.__init__",
            //
            "somesillypackage.child3.__init__",
            //
            "somesillypackage.child4.__init__",
            //
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .descendant_modules("somesillypackage.child1")
            .unwrap(),
        hashset! {
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.a",
            "somesillypackage.child1.b",
            "somesillypackage.child1.c",
            "somesillypackage.child1.d",
            "somesillypackage.child1.e",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .descendant_modules("somesillypackage.child2")
            .unwrap(),
        hashset! {
            "somesillypackage.child2.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .descendant_modules("somesillypackage.child3")
            .unwrap(),
        hashset! {
            "somesillypackage.child3.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .descendant_modules("somesillypackage.child4")
            .unwrap(),
        hashset! {
            "somesillypackage.child4.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .descendant_modules("somesillypackage.child5")
            .unwrap(),
        hashset! {
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_direct_imports() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph.direct_imports(),
        hashmap! {
            "somesillypackage.__init__" => hashset!{
                "somesillypackage.a",
                "somesillypackage.child1.a",
                "somesillypackage.b",
                "somesillypackage.child1.b",
                "somesillypackage.c",
                "somesillypackage.child1.c",
                "somesillypackage.d",
                "somesillypackage.child1.d",
                "somesillypackage.e",
                "somesillypackage.child1.e",
                "somesillypackage.child1.__init__",
                "somesillypackage.child2.__init__",
                "somesillypackage.child3.__init__",
                "somesillypackage.child4.__init__",
                "somesillypackage.child5.__init__",
            },
            "somesillypackage.a" => hashset!{
                "somesillypackage.b",
                "somesillypackage.c",
            },
            "somesillypackage.b" => hashset!{
                "somesillypackage.c",
            },
            "somesillypackage.c" => hashset!{
                "somesillypackage.d",
                "somesillypackage.e",
            },
            "somesillypackage.d" => hashset!{
                "somesillypackage.e"
            },
            "somesillypackage.e" => hashset!{},
            "somesillypackage.z" => hashset! {
                "somesillypackage.a",
                "somesillypackage.child1.a",
                "somesillypackage.b",
                "somesillypackage.child1.b",
                "somesillypackage.c",
                "somesillypackage.child1.c",
                "somesillypackage.d",
                "somesillypackage.child1.d",
                "somesillypackage.e",
                "somesillypackage.child1.e",
                "somesillypackage.child1.__init__",
                "somesillypackage.child2.__init__",
                "somesillypackage.child3.__init__",
                "somesillypackage.child4.__init__",
                "somesillypackage.child5.__init__",
            },
            "somesillypackage.child1.__init__" => hashset!{
                "somesillypackage.a",
                "somesillypackage.child1.a",
                "somesillypackage.b",
                "somesillypackage.child1.b",
                "somesillypackage.c",
                "somesillypackage.child1.c",
                "somesillypackage.d",
                "somesillypackage.child1.d",
                "somesillypackage.e",
                "somesillypackage.child1.e",
                "somesillypackage.__init__",
                "somesillypackage.child2.__init__",
                "somesillypackage.child3.__init__",
                "somesillypackage.child4.__init__",
                "somesillypackage.child5.__init__",
            },
            "somesillypackage.child1.a" => hashset!{},
            "somesillypackage.child1.b" => hashset!{},
            "somesillypackage.child1.c" => hashset!{},
            "somesillypackage.child1.d" => hashset!{},
            "somesillypackage.child1.e" => hashset!{},
            "somesillypackage.child1.z" => hashset!{
                "somesillypackage.a",
                "somesillypackage.child1.a",
                "somesillypackage.b",
                "somesillypackage.child1.b",
                "somesillypackage.c",
                "somesillypackage.child1.c",
                "somesillypackage.d",
                "somesillypackage.child1.d",
                "somesillypackage.e",
                "somesillypackage.child1.e",
                "somesillypackage.__init__",
                "somesillypackage.child2.__init__",
                "somesillypackage.child3.__init__",
                "somesillypackage.child4.__init__",
                "somesillypackage.child5.__init__",
            },
            "somesillypackage.child2.__init__" => hashset!{},
            "somesillypackage.child3.__init__" => hashset!{},
            "somesillypackage.child4.__init__" => hashset!{},
            "somesillypackage.child5.__init__" => hashset!{},
        }
        .into_iter()
        .map(|(k, v)| (
            k.to_string(),
            v.into_iter().map(|v| v.to_string()).collect()
        ))
        .collect()
    );

    assert_eq!(import_graph.direct_imports_flat().len(), 66);
    assert!(import_graph.direct_imports_flat().contains(&(
        "somesillypackage.child1.z".to_string(),
        "somesillypackage.child1.e".to_string()
    )));
}

#[test]
fn test_direct_import_exists() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert!(import_graph
        .direct_import_exists("somesillypackage.a", "somesillypackage.b")
        .unwrap());
    assert!(!import_graph
        .direct_import_exists("somesillypackage.b", "somesillypackage.a")
        .unwrap());
}

#[test]
fn test_modules_directly_imported_by_module() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .modules_directly_imported_by("somesillypackage.__init__")
            .unwrap(),
        hashset! {
            "somesillypackage.a",
            "somesillypackage.child1.a",
            "somesillypackage.b",
            "somesillypackage.child1.b",
            "somesillypackage.c",
            "somesillypackage.child1.c",
            "somesillypackage.d",
            "somesillypackage.child1.d",
            "somesillypackage.e",
            "somesillypackage.child1.e",
            "somesillypackage.child1.__init__",
            "somesillypackage.child2.__init__",
            "somesillypackage.child3.__init__",
            "somesillypackage.child4.__init__",
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .modules_directly_imported_by("somesillypackage.z")
            .unwrap(),
        hashset! {
            "somesillypackage.a",
            "somesillypackage.child1.a",
            "somesillypackage.b",
            "somesillypackage.child1.b",
            "somesillypackage.c",
            "somesillypackage.child1.c",
            "somesillypackage.d",
            "somesillypackage.child1.d",
            "somesillypackage.e",
            "somesillypackage.child1.e",
            "somesillypackage.child1.__init__",
            "somesillypackage.child2.__init__",
            "somesillypackage.child3.__init__",
            "somesillypackage.child4.__init__",
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .modules_directly_imported_by("somesillypackage.child1.__init__")
            .unwrap(),
        hashset! {
            "somesillypackage.a",
            "somesillypackage.child1.a",
            "somesillypackage.b",
            "somesillypackage.child1.b",
            "somesillypackage.c",
            "somesillypackage.child1.c",
            "somesillypackage.d",
            "somesillypackage.child1.d",
            "somesillypackage.e",
            "somesillypackage.child1.e",
            "somesillypackage.__init__",
            "somesillypackage.child2.__init__",
            "somesillypackage.child3.__init__",
            "somesillypackage.child4.__init__",
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .modules_directly_imported_by("somesillypackage.child1.z")
            .unwrap(),
        hashset! {
            "somesillypackage.a",
            "somesillypackage.child1.a",
            "somesillypackage.b",
            "somesillypackage.child1.b",
            "somesillypackage.c",
            "somesillypackage.child1.c",
            "somesillypackage.d",
            "somesillypackage.child1.d",
            "somesillypackage.e",
            "somesillypackage.child1.e",
            "somesillypackage.__init__",
            "somesillypackage.child2.__init__",
            "somesillypackage.child3.__init__",
            "somesillypackage.child4.__init__",
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_modules_directly_imported_by_package() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .modules_directly_imported_by("somesillypackage")
            .unwrap(),
        hashset! {
            "somesillypackage.a",
            "somesillypackage.child1.a",
            "somesillypackage.b",
            "somesillypackage.child1.b",
            "somesillypackage.c",
            "somesillypackage.child1.c",
            "somesillypackage.d",
            "somesillypackage.child1.d",
            "somesillypackage.e",
            "somesillypackage.child1.e",
            "somesillypackage.__init__",  // coming from somesillypackage.child1.__init__
            "somesillypackage.child1.__init__",
            "somesillypackage.child2.__init__",
            "somesillypackage.child3.__init__",
            "somesillypackage.child4.__init__",
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .modules_directly_imported_by("somesillypackage.child1")
            .unwrap(),
        hashset! {
            "somesillypackage.a",
            "somesillypackage.child1.a",
            "somesillypackage.b",
            "somesillypackage.child1.b",
            "somesillypackage.c",
            "somesillypackage.child1.c",
            "somesillypackage.d",
            "somesillypackage.child1.d",
            "somesillypackage.e",
            "somesillypackage.child1.e",
            "somesillypackage.__init__",
            "somesillypackage.child2.__init__",
            "somesillypackage.child3.__init__",
            "somesillypackage.child4.__init__",
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_modules_that_directly_import_module() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .modules_that_directly_import("somesillypackage.__init__")
            .unwrap(),
        hashset! {
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .modules_that_directly_import("somesillypackage.a")
            .unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .modules_that_directly_import("somesillypackage.child1.__init__")
            .unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .modules_that_directly_import("somesillypackage.child1.a")
            .unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .modules_that_directly_import("somesillypackage.child2.__init__")
            .unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_modules_that_directly_import_package() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .modules_that_directly_import("somesillypackage")
            .unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
            "somesillypackage.a",
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.d",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .modules_that_directly_import("somesillypackage.child1")
            .unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .modules_that_directly_import("somesillypackage.child2")
            .unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_downstream_modules_of_module() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .downstream_modules("somesillypackage.a")
            .unwrap(),
        hashset! {
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.d",
            "somesillypackage.e",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .downstream_modules("somesillypackage.c")
            .unwrap(),
        hashset! {
            "somesillypackage.d",
            "somesillypackage.e",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .downstream_modules("somesillypackage.e")
            .unwrap(),
        hashset! {}
    );
}

#[test]
fn test_downstream_modules_of_package() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph.downstream_modules("somesillypackage").unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.a",
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.d",
            "somesillypackage.e",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.a",
            "somesillypackage.child1.b",
            "somesillypackage.child1.c",
            "somesillypackage.child1.d",
            "somesillypackage.child1.e",
            "somesillypackage.child2.__init__",
            "somesillypackage.child3.__init__",
            "somesillypackage.child4.__init__",
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .downstream_modules("somesillypackage.child1")
            .unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.a",
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.d",
            "somesillypackage.e",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.a",
            "somesillypackage.child1.b",
            "somesillypackage.child1.c",
            "somesillypackage.child1.d",
            "somesillypackage.child1.e",
            "somesillypackage.child2.__init__",
            "somesillypackage.child3.__init__",
            "somesillypackage.child4.__init__",
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .downstream_modules("somesillypackage.child2")
            .unwrap(),
        hashset! {}
    );
}

#[test]
fn test_upstream_modules_of_module() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph.upstream_modules("somesillypackage.a").unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph.upstream_modules("somesillypackage.c").unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
            "somesillypackage.a",
            "somesillypackage.b",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph.upstream_modules("somesillypackage.e").unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
            "somesillypackage.a",
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.d",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_upstream_modules_of_package() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph.upstream_modules("somesillypackage").unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.a",
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.d",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .upstream_modules("somesillypackage.child1")
            .unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph
            .upstream_modules("somesillypackage.child2")
            .unwrap(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_shortest_path() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .shortest_path("somesillypackage.a", "somesillypackage.e")
            .unwrap()
            .unwrap(),
        vec![
            "somesillypackage.a",
            "somesillypackage.c",
            "somesillypackage.e"
        ],
    );
    assert!(import_graph
        .shortest_path("somesillypackage.e", "somesillypackage.a")
        .is_ok_and(|shortest_path| shortest_path.is_none()));
}

#[test]
fn test_path_exists() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert!(import_graph
        .path_exists("somesillypackage.a", "somesillypackage.e")
        .unwrap(),);
    assert!(!import_graph
        .path_exists("somesillypackage.e", "somesillypackage.a")
        .unwrap(),);
}

#[test]
fn test_path_exists_packages() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert!(import_graph
        .path_exists("somesillypackage.child1", "somesillypackage.child2")
        .unwrap(),);
    assert!(!import_graph
        .path_exists("somesillypackage.child2", "somesillypackage.child1")
        .unwrap(),);
}

#[test]
fn test_ignore_imports() {
    let root_package_path = Path::new("./testpackages/somesillypackage");

    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .shortest_path("somesillypackage.a", "somesillypackage.e")
            .unwrap()
            .unwrap(),
        vec![
            "somesillypackage.a",
            "somesillypackage.c",
            "somesillypackage.e"
        ],
    );

    let import_graph = import_graph
        .ignore_imports([("somesillypackage.a", "somesillypackage.c")])
        .unwrap();
    assert_eq!(
        import_graph
            .shortest_path("somesillypackage.a", "somesillypackage.e")
            .unwrap()
            .unwrap(),
        vec![
            "somesillypackage.a",
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.e"
        ],
    );

    let import_graph = import_graph
        .ignore_imports([("somesillypackage.a", "somesillypackage.b")])
        .unwrap();
    assert!(import_graph
        .shortest_path("somesillypackage.a", "somesillypackage.e")
        .unwrap()
        .is_none());

    let result = import_graph.ignore_imports([("somesillypackage.a", "somesillypackage.b")]);
    assert!(matches!(
        result.unwrap_err().downcast::<Error>().unwrap(),
        Error::ImportNotFound(_, _)
    ));
}

#[test]
fn test_subgraph() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    let subgraph = import_graph.subgraph("somesillypackage.child1").unwrap();
    assert_eq!(
        subgraph.packages(),
        hashset! {
            "somesillypackage.child1".to_string(),
        }
    );
    assert_eq!(
        subgraph.modules(),
        hashset! {
            "somesillypackage.child1.__init__",
            "somesillypackage.child1.a",
            "somesillypackage.child1.b",
            "somesillypackage.child1.c",
            "somesillypackage.child1.d",
            "somesillypackage.child1.e",
            "somesillypackage.child1.z",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        subgraph.direct_imports(),
        hashmap! {
            "somesillypackage.child1.__init__" => hashset!{
                "somesillypackage.child1.a",
                "somesillypackage.child1.b",
                "somesillypackage.child1.c",
                "somesillypackage.child1.d",
                "somesillypackage.child1.e",
            },
            "somesillypackage.child1.a" => hashset!{},
            "somesillypackage.child1.b" => hashset!{},
            "somesillypackage.child1.c" => hashset!{},
            "somesillypackage.child1.d" => hashset!{},
            "somesillypackage.child1.e" => hashset!{},
            "somesillypackage.child1.z" => hashset!{
                "somesillypackage.child1.a",
                "somesillypackage.child1.b",
                "somesillypackage.child1.c",
                "somesillypackage.child1.d",
                "somesillypackage.child1.e",
            },
        }
        .into_iter()
        .map(|(k, v)| (
            k.to_string(),
            v.into_iter().map(|v| v.to_string()).collect()
        ))
        .collect()
    );
}

#[test]
fn test_squash_package() {
    let root_package_path = Path::new("./testpackages/somesillypackage");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    let squashed = import_graph
        .squash_package("somesillypackage.child1")
        .unwrap();
    assert_eq!(
        squashed.packages(),
        hashset! {
            "somesillypackage",
            "somesillypackage.child1",
            "somesillypackage.child2",
            "somesillypackage.child3",
            "somesillypackage.child4",
            "somesillypackage.child5",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        squashed.modules(),
        hashset! {
            "somesillypackage.__init__",
            "somesillypackage.a",
            "somesillypackage.b",
            "somesillypackage.c",
            "somesillypackage.d",
            "somesillypackage.e",
            "somesillypackage.z",
            "somesillypackage.child1.__init__",
            "somesillypackage.child2.__init__",
            "somesillypackage.child3.__init__",
            "somesillypackage.child4.__init__",
            "somesillypackage.child5.__init__",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        squashed.direct_imports(),
        hashmap! {
            "somesillypackage.__init__" => hashset!{
                "somesillypackage.a",
                "somesillypackage.b",
                "somesillypackage.c",
                "somesillypackage.d",
                "somesillypackage.e",
                "somesillypackage.child1.__init__",
                "somesillypackage.child2.__init__",
                "somesillypackage.child3.__init__",
                "somesillypackage.child4.__init__",
                "somesillypackage.child5.__init__",
            },
            "somesillypackage.a" => hashset!{
                "somesillypackage.b",
                "somesillypackage.c",
            },
            "somesillypackage.b" => hashset!{
                "somesillypackage.c",
            },
            "somesillypackage.c" => hashset!{
                "somesillypackage.d",
                "somesillypackage.e",
            },
            "somesillypackage.d" => hashset!{
                "somesillypackage.e"
            },
            "somesillypackage.e" => hashset!{},
            "somesillypackage.z" => hashset! {
                "somesillypackage.a",
                "somesillypackage.b",
                "somesillypackage.c",
                "somesillypackage.d",
                "somesillypackage.e",
                "somesillypackage.child1.__init__",
                "somesillypackage.child2.__init__",
                "somesillypackage.child3.__init__",
                "somesillypackage.child4.__init__",
                "somesillypackage.child5.__init__",
            },
            "somesillypackage.child1.__init__" => hashset!{
                "somesillypackage.a",
                "somesillypackage.b",
                "somesillypackage.c",
                "somesillypackage.d",
                "somesillypackage.e",
                "somesillypackage.__init__",
                "somesillypackage.child2.__init__",
                "somesillypackage.child3.__init__",
                "somesillypackage.child4.__init__",
                "somesillypackage.child5.__init__",
            },
            "somesillypackage.child2.__init__" => hashset!{},
            "somesillypackage.child3.__init__" => hashset!{},
            "somesillypackage.child4.__init__" => hashset!{},
            "somesillypackage.child5.__init__" => hashset!{},
        }
        .into_iter()
        .map(|(k, v)| (
            k.to_string(),
            v.into_iter().map(|v| v.to_string()).collect()
        ))
        .collect()
    );
}
