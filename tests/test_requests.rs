use maplit::hashset;
use std::path::Path;

use pyimports::ImportGraphBuilder;

#[test]
fn test_modules_directly_imported_by() {
    let root_package_path = Path::new("./testpackages/requests");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .modules_directly_imported_by("requests.__init__")
            .unwrap(),
        hashset! {
            "requests.__version__",
            "requests.api",
            "requests.exceptions",
            "requests.models",
            "requests.packages",
            "requests.sessions",
            "requests.status_codes",
            "requests.utils",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    )
}

#[test]
fn test_downstream_modules() {
    let root_package_path = Path::new("./testpackages/requests");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .downstream_modules("requests.__init__")
            .unwrap(),
        hashset! {
            "requests.__version__",
            "requests._internal_utils",
            "requests.adapters",
            "requests.api",
            "requests.auth",
            "requests.certs",
            "requests.compat",
            "requests.cookies",
            "requests.exceptions",
            "requests.hooks",
            "requests.models",
            "requests.packages",
            "requests.sessions",
            "requests.status_codes",
            "requests.structures",
            "requests.utils",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
    assert_eq!(
        import_graph.downstream_modules("requests.utils").unwrap(),
        hashset! {
            "requests.__version__",
            "requests._internal_utils",
            "requests.certs",
            "requests.compat",
            "requests.cookies",
            "requests.exceptions",
            "requests.structures",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    );
}

#[test]
fn test_shortest_path() {
    let root_package_path = Path::new("./testpackages/requests");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert!(
        // There are 3 equally short paths.
        hashset! {
            vec!["requests.__init__", "requests.models", "requests.cookies",].into_iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            vec!["requests.__init__", "requests.sessions", "requests.cookies",].into_iter().map(|s| s.to_string()).collect(),
            vec!["requests.__init__", "requests.utils", "requests.cookies",].into_iter().map(|s| s.to_string()).collect(),
        }
        .contains(
            &import_graph
                .shortest_path("requests.__init__", "requests.cookies")
                .unwrap()
                .unwrap()
        )
    );
}
