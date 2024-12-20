use anyhow::Result;
use std::path::Path;

pub fn path_to_pypath(path: &Path, root_path: &Path) -> Result<String> {
    let path = path.strip_prefix(root_path.parent().unwrap())?;
    let mut s = path.to_str().unwrap();
    if s.ends_with(".py") {
        s = s.strip_suffix(".py").unwrap();
    }
    Ok(s.replace("/", "."))
}
