use anyhow::Result;
use std::time;
use std::{env, path};

use pyimports::{import_discovery, package_discovery};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let root_path: path::PathBuf = args[1].clone().into();

    let package_info = timeit("Package discovery", || {
        package_discovery::PackageInfo::build(&root_path)
    })?;
    println!("{} items", package_info.queries().get_all_items().count());

    let imports_info = timeit("Import discovery", || {
        import_discovery::ImportsInfo::build_from_package_info(&package_info)
    })?;

    Ok(())
}

fn timeit<F: Fn() -> Result<T>, T>(s: &str, f: F) -> Result<T> {
    let instance = time::Instant::now();
    let t = f()?;
    println!("{} took {:?}", s, instance.elapsed());
    Ok(t)
}
