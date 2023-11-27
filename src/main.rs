use anyhow::Result;
use std::{env, path::Path};

use pyimports::ImportGraphBuilder;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let root_package_path = Path::new(&args[1]);

    let import_graph = {
        let mut import_graph = ImportGraphBuilder::new(root_package_path)
            .use_cache()
            .build()?;
        if args.len() == 2 {
            import_graph
        } else {
            import_graph = import_graph.subgraph(&args[2])?;
            // for child_package in import_graph.child_packages(&args[2])? {
            //     import_graph = import_graph.squash_package(&child_package)?;
            // }
            import_graph
        }
    };

    println!("digraph {{");
    println!("    concentrate=true;");
    for module in import_graph.modules() {
        println!("    \"{}\";", module);
    }
    for (from_module, to_module) in import_graph.direct_imports_flat() {
        println!("    \"{}\" -> \"{}\";", from_module, to_module);
    }
    println!("}}");

    Ok(())
}
