use pyimports::contracts::layers::{Layer, LayeredArchitectureContract};
use pyimports::contracts::ImportsContract;
use pyimports::grimp_compare::build_imports_info;
use pyimports::imports_info::ImportsInfo;
use pyimports::package_info::PackageItemToken;

fn main() {
    divan::main();
}

fn _item(imports_info: &ImportsInfo, pypath: &str) -> PackageItemToken {
    imports_info
        .package_info()
        .get_item_by_pypath(&pypath.parse().unwrap())
        .unwrap()
        .token()
}

#[divan::bench]
fn benchmark_top_level_layers_large_graph(bencher: divan::Bencher) {
    let imports_info = build_imports_info("./data/large_graph.json").unwrap();

    let contract = LayeredArchitectureContract::new(&[
        Layer::new([_item(&imports_info, "mypackage.data")], true),
        Layer::new([_item(&imports_info, "mypackage.domain")], true),
        Layer::new([_item(&imports_info, "mypackage.application")], true),
        Layer::new([_item(&imports_info, "mypackage.plugins")], true),
    ])
    .with_deep_imports_allowed();

    bencher.bench(|| contract.verify(&imports_info).unwrap());
}

#[divan::bench]
fn benchmark_deep_layers_large_graph(bencher: divan::Bencher) {
    let imports_info = build_imports_info("./data/large_graph.json").unwrap();

    let contract = LayeredArchitectureContract::new(&[
        Layer::new([_item(&imports_info, "mypackage.plugins.5634303718.1007553798.8198145119.application.3242334296.2454157946")], true),
        Layer::new([_item(&imports_info, "mypackage.plugins.5634303718.1007553798.8198145119.application.3242334296.5033127033")], true),
        Layer::new([_item(&imports_info, "mypackage.plugins.5634303718.1007553798.8198145119.application.3242334296.9089085203")], true),
        Layer::new([_item(&imports_info, "mypackage.plugins.5634303718.1007553798.8198145119.application.3242334296.1752284225")], true),
        Layer::new([_item(&imports_info, "mypackage.plugins.5634303718.1007553798.8198145119.application.3242334296.1693068682")], true),
        Layer::new([_item(&imports_info, "mypackage.plugins.5634303718.1007553798.8198145119.application.3242334296.6666171185")], true),
        Layer::new([_item(&imports_info, "mypackage.plugins.5634303718.1007553798.8198145119.application.3242334296.9009030339")], true),
        Layer::new([_item(&imports_info, "mypackage.plugins.5634303718.1007553798.8198145119.application.3242334296.6397984863")], true),
        Layer::new([_item(&imports_info, "mypackage.plugins.5634303718.1007553798.8198145119.application.3242334296.1991886645")], true),
    ]).with_deep_imports_allowed();

    bencher.bench(|| contract.verify(&imports_info).unwrap());
}
