use maplit::hashset;
use pretty_assertions::assert_eq;
use std::path::Path;

use pyimports::ImportGraphBuilder;

#[test]
fn test_modules_directly_imported_by() {
    let root_package_path = Path::new("./testpackages/django");
    let import_graph = ImportGraphBuilder::new(root_package_path).build().unwrap();
    assert_eq!(
        import_graph
            .modules_directly_imported_by("django.shortcuts")
            .unwrap(),
        hashset! {
            "django.http.__init__",
            "django.template.loader",
            "django.urls.__init__",
            "django.utils.functional",
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    )
}
