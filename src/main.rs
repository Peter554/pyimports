use anyhow::Result;
use std::{env, path::Path};

use pyimports::ImportGraphBuilder;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let root_package_path = Path::new(&args[1]);
    let import_graph = ImportGraphBuilder::new(root_package_path).build()?;
    dbg!(import_graph.modules().len());
    Ok(())
}
