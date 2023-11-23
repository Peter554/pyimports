use anyhow::Result;
use std::{env, path::Path};

use pyimports::import_discovery;
use pyimports::indexing;
use pyimports::package_discovery;

pub fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let root_package_path = Path::new(&args[1]);

    let package = package_discovery::discover_package(root_package_path)?;
    let packages_by_pypath = indexing::get_packages_by_pypath(&package)?;
    let modules_by_pypath = indexing::get_modules_by_pypath(&package)?;
    let imports = import_discovery::discover_imports(&modules_by_pypath)?;

    // dbg!(&package);
    // dbg!(count_modules(&package));

    // dbg!(&imports);
    dbg!(imports.get("octoenergy.plugins.territories.ita.billing.tasks"));

    Ok(())
}

fn count_modules(package: &package_discovery::Package) -> usize {
    let mut count = 0;
    count += package.modules.len();
    for child in package.children.iter() {
        count += count_modules(child);
    }
    count
}
