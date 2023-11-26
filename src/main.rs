use anyhow::Result;
use std::{env, path::Path};

use pyimports::ImportGraphBuilder;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let root_package_path = Path::new(&args[1]);

    let import_graph = {
        let mut import_graph = ImportGraphBuilder::new(root_package_path).build()?;
        if args.len() == 2 {
            import_graph
        } else {
            for child_package in import_graph.child_packages(&args[2])? {
                import_graph = import_graph.squash_package(&child_package)?;
            }
            import_graph.subgraph(&args[2])?
        }
    };

    let imports = import_graph.direct_imports_flat();
    println!("source, target");
    for (from_module, to_module) in imports {
        println!("{}, {}", from_module, to_module);
    }

    Ok(())
}
