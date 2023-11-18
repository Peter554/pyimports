use std::{fs, path::Path};

#[derive(Debug)]
pub struct Package {
    pub pypath: String,
    pub children: Vec<Package>,
    pub modules: Vec<Module>,
}

impl Package {
    fn new(pypath: String) -> Self {
        Package {
            pypath,
            children: vec![],
            modules: vec![],
        }
    }

    fn add_child(&mut self, child: Package) {
        self.children.push(child);
    }

    fn add_module(&mut self, module: Module) {
        self.modules.push(module);
    }
}

#[derive(Debug)]
pub struct Module {
    pub pypath: String,
}

impl Module {
    fn new(pypath: String) -> Self {
        Module { pypath }
    }
}

pub fn discover_package(package_path: &Path) -> Package {
    _discover_package(package_path, package_path).unwrap()
}

fn _discover_package(root_package_path: &Path, package_path: &Path) -> Option<Package> {
    let mut is_package = false;
    let mut files = vec![];
    let mut dirs = vec![];

    let entries = fs::read_dir(package_path).unwrap().collect::<Vec<_>>();
    for entry in entries {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        if file_type.is_file() {
            let file_name = entry.file_name();
            files.push(entry);
            if file_name == "__init__.py" {
                is_package = true;
            }
        } else if file_type.is_dir() {
            dirs.push(entry);
        }
    }

    if !is_package {
        return None;
    }

    let pypath = get_pypath(root_package_path, package_path, false);
    let mut package = Package::new(pypath);
    for file in files {
        if file.path().extension().unwrap_or_default() != "py" {
            continue;
        }
        let pypath = get_pypath(root_package_path, &file.path(), true);
        let module = Module::new(pypath);
        package.add_module(module);
    }
    for dir in dirs {
        if dir.file_name().to_str().unwrap().starts_with(".") {
            continue;
        }
        if let Some(child) = _discover_package(root_package_path, &dir.path()) {
            package.add_child(child);
        }
    }
    Some(package)
}

fn get_pypath(root_package_path: &Path, path: &Path, is_file: bool) -> String {
    let root_path_part = root_package_path.file_name().unwrap().to_str().unwrap();

    let mut path_part = path
        .strip_prefix(root_package_path)
        .unwrap()
        .to_str()
        .unwrap();
    if is_file {
        path_part = path_part.strip_suffix(".py").unwrap();
    }

    if path_part.is_empty() {
        root_path_part.to_string()
    } else {
        format!("{root_path_part}.{path_part}").replace("/", ".")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_discover_package() {
        let root_package_path = Path::new("./example");

        let root_package = discover_package(root_package_path);

        assert_eq!(root_package.pypath, "example");
        let root_modules = root_package
            .modules
            .iter()
            .map(|m| m.pypath.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            root_modules,
            ["example.__init__", "example.a", "example.b",]
                .into_iter()
                .collect::<HashSet<_>>()
        );
        assert_eq!(root_package.children.len(), 1);

        let child_package = root_package.children.first().unwrap();

        assert_eq!(child_package.pypath, "example.child");
        let root_modules = child_package
            .modules
            .iter()
            .map(|m| m.pypath.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            root_modules,
            [
                "example.child.__init__",
                "example.child.c",
                "example.child.d",
            ]
            .into_iter()
            .collect::<HashSet<_>>()
        );
        assert_eq!(child_package.children.len(), 0);
    }
}
