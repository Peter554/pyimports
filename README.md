# pyimports

[![CI](https://github.com/Peter554/pyimports/actions/workflows/ci.yml/badge.svg)](https://github.com/Peter554/pyimports/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/pyimports.svg)](https://crates.io/crates/pyimports)

A rust crate for parsing and analyzing the imports within a python package.

```rust
use anyhow::Result;

use pyimports::{testpackage,TestPackage};
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
