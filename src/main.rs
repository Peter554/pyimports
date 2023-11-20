use std::{env, path::Path};
use anyhow::Result;

use pyimports::package_discovery::{self, Package};

pub fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let root_package_path = Path::new(&args[1]);

    let package = package_discovery::discover_package(root_package_path)?;

    // dbg!(&package);
    dbg!(count_modules(&package));

    Ok(())
}

fn count_modules(package: &Package) -> usize {
    let mut count = 0;
    count += package.modules.len();
    for child in package.children.iter() {
        count += count_modules(child);
    }
    count
}
