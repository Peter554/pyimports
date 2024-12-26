use anyhow::Result;
use std::time;
use std::{env, path};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let root_path: path::PathBuf = args[1].clone().into();

    let _package_info = timeit("Package discovery", || {
        pyimports::PackageInfo::build(&root_path)
    })?;
    println!("{} items", _package_info.get_all_items().count());

    let _imports_info = timeit("Import discovery", || {
        pyimports::ImportsInfo::build(_package_info.clone())
    })?;

    Ok(())
}

fn timeit<F: Fn() -> Result<T>, T>(s: &str, f: F) -> Result<T> {
    let instance = time::Instant::now();
    let t = f()?;
    println!("{} took {:?}", s, instance.elapsed());
    Ok(t)
}
