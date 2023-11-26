use maplit::hashset;
use std::path::Path;

use pyimports::ImportGraphBuilder;

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
