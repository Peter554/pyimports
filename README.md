# pyimports

[![CI](https://github.com/Peter554/pyimports/actions/workflows/ci.yml/badge.svg)](https://github.com/Peter554/pyimports/actions/workflows/ci.yml)

A rust crate for parsing and analyzing the imports within a python package.

- [Docs](https://docs.rs/pyimports/0.1.0/pyimports/).

```rust
use anyhow::Result;

use pyimports::{testpackage,testutils::TestPackage};
use pyimports::{PackageInfo,ImportsInfo};

fn main() -> Result<()> {
    let testpackage = testpackage! {
        "__init__.py" => "",
        "a.py" => ""
    };

    let package_info = PackageInfo::build(testpackage.path())?;
    let imports_info = ImportsInfo::build(package_info)?;

    Ok(())
}
```
