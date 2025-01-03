# pyimports

[![CI](https://github.com/Peter554/pyimports/actions/workflows/ci.yml/badge.svg)](https://github.com/Peter554/pyimports/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/pyimports.svg)](https://crates.io/crates/pyimports)
[![Docs](https://img.shields.io/badge/Docs-grey)](https://docs.rs/pyimports/)

A rust crate for parsing and analyzing the imports within a python package.

## Example

A short example (for more information refer to [the docs](https://docs.rs/pyimports/)):

```rust
use anyhow::Result;
use maplit::{hashmap,hashset};

use pyimports::prelude::*;
use pyimports::{PackageInfo,ImportsInfo,PackageItemToken,InternalImportsPathQuery};

// You shouldn't use `testpackage!`, it just creates a fake python package
// in a temporary directory. It's (unfortunately) included in the public API
// so that it can be used in the doctests.
use pyimports::{testpackage,TestPackage};

fn main() -> Result<()> {
    let testpackage = testpackage! {
        "__init__.py" => "from testpackage import a, b",
        "a.py" => "from testpackage import b",
        "b.py" => "from testpackage import c, d",
        "c.py" => "from testpackage import d",
        "d.py" => ""
    };
    let package_info = PackageInfo::build(testpackage.path())?;
    let imports_info = ImportsInfo::build(package_info)?;

    let item = |pypath: &str| -> Result<PackageItemToken> {
        Ok(imports_info.package_info().get_item_by_pypath(pypath)?.unwrap().token())
    };

    let root_pkg = item("testpackage")?;
    let root_init = item("testpackage.__init__")?;
    let a = item("testpackage.a")?;
    let b = item("testpackage.b")?;
    let c = item("testpackage.c")?;
    let d = item("testpackage.d")?;

    assert_eq!(
        imports_info.internal_imports().get_direct_imports(),
        hashmap! {
            root_pkg => hashset!{root_init},
            root_init => hashset!{a, b},
            a => hashset!{b},
            b => hashset!{c, d},
            c => hashset!{d},
            d => hashset!{},
        }
    );

    assert_eq!(
        imports_info.internal_imports().get_downstream_items(root_pkg)?,
        hashset! {root_init, a, b, c, d}
    );

    assert_eq!(
        imports_info.internal_imports().find_path(
            &InternalImportsPathQuery::new()
                .from(root_pkg)
                .to(d)
        )?,
        Some(vec![root_pkg, root_init, b, d])
    );

    Ok(())
}
```

## Scope

This crate might be useful for something eventually, but right now it's mainly just
a hobby project for me to learn about rust.

If you are looking for something more mature, try [grimp](https://github.com/seddonym/grimp/)/[import-linter](https://github.com/seddonym/import-linter).

## Limitations

The python parser used within this crate does not currently support python 3.12+ - see the related GitHub issue [here](https://github.com/RustPython/Parser/issues/125).

## Next steps

Some possible next steps that I may explore if/when I get time:

- Fix issue with python parser, to support python 3.12+.
- Performance benchmarking/improvements.
- Python bindings (via [maturin](https://github.com/PyO3/maturin)).
- Higher level features e.g. import contracts, similar to [import-linter](https://github.com/seddonym/import-linter).
- Faster path calculations (via e.g. [fast_paths](https://github.com/easbar/fast_paths)).
