use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use pyimports::ImportGraphBuilder;
use serde::Serialize;

#[derive(Parser)]
struct Cli {
    root_package_path: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Export {
        #[arg(long)]
        exclude_type_checking_imports: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Export {
            exclude_type_checking_imports,
        }) => export(&cli.root_package_path, exclude_type_checking_imports),
        _ => Ok(()),
    }
}

fn export(root_package_path: &Path, exclude_type_checking_imports: bool) -> Result<()> {
    let mut builder = ImportGraphBuilder::new(root_package_path).use_cache();
    if exclude_type_checking_imports {
        builder = builder.exclude_type_checking_imports();
    }
    let graph = builder.build()?;

    let packages = graph.packages().into_iter().map(|pypath| Package {
        __type__: "package".to_string(),
        pypath,
    });
    let modules = graph.modules().into_iter().map(|pypath| Module {
        __type__: "module".to_string(),
        pypath: pypath.clone(),
        package_pypath: graph.package_from_module(&pypath).unwrap(),
    });
    let imports =
        graph
            .direct_imports_flat()
            .into_iter()
            .map(|(from_module_pypath, to_module_pypath)| {
                let import_metadata = graph
                    .import_metadata(&from_module_pypath, &to_module_pypath)
                    .unwrap();
                Import {
                    __type__: "import".to_string(),
                    from_module_pypath,
                    to_module_pypath,
                    line_number: import_metadata.map(|m| m.line_number),
                }
            });

    for package in packages {
        println!("{}", serde_json::to_string(&package)?);
    }
    for module in modules {
        println!("{}", serde_json::to_string(&module)?);
    }
    for import in imports {
        println!("{}", serde_json::to_string(&import)?);
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct Package {
    __type__: String,
    pypath: String,
}

#[derive(Debug, Serialize)]
struct Module {
    __type__: String,
    pypath: String,
    package_pypath: String,
}

#[derive(Debug, Serialize)]
struct Import {
    __type__: String,
    from_module_pypath: String,
    to_module_pypath: String,
    line_number: Option<u32>,
}
