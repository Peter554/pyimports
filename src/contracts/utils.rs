use crate::contracts::{ContractViolation, ForbiddenExternalImport, ForbiddenInternalImport};
use crate::imports_info::{
    ExternalImportsPathQueryBuilder, ImportsInfo, InternalImportsPathQueryBuilder,
};
use crate::package_info::PackageItemToken;
use crate::prelude::*;
use crate::pypath::Pypath;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashSet;
use tap::prelude::*;

pub(super) fn ignore_imports(
    imports_info: &ImportsInfo,
    ignored_internal_imports: &[(PackageItemToken, PackageItemToken)],
    ignored_external_imports: &[(PackageItemToken, Pypath)],
    ignore_typechecking_imports: bool,
) -> Result<ImportsInfo> {
    // Assumption: It's best/reasonable to clone here and remove the ignored imports from the graph.
    // An alternative could be to ignore the imports dynamically via a new field on `InternalImportsPathQuery`.
    let mut imports_info = imports_info.clone();
    imports_info.remove_imports(
        ignored_internal_imports.to_owned(),
        ignored_external_imports.to_owned(),
    )?;
    if ignore_typechecking_imports {
        imports_info.remove_typechecking_imports()?;
    }
    Ok(imports_info)
}
pub(super) fn find_internal_import_violations(
    forbidden_imports: &[ForbiddenInternalImport],
    imports_info: &ImportsInfo,
) -> Result<Vec<ContractViolation>> {
    let violations = forbidden_imports
        .into_par_iter()
        .try_fold(
            Vec::new,
            |mut violations, forbidden_import| -> anyhow::Result<_> {
                // A contract operates in "as packages" mode, meaning
                // items are expanded to include their descendants.
                let from = forbidden_import
                    .from
                    .conv::<HashSet<PackageItemToken>>()
                    .with_descendants(imports_info.package_info());
                let to = forbidden_import
                    .to
                    .conv::<HashSet<PackageItemToken>>()
                    .with_descendants(imports_info.package_info());
                let except_via = forbidden_import
                    .except_via()
                    .clone()
                    .with_descendants(imports_info.package_info());

                let path = imports_info.internal_imports().find_path(
                    &InternalImportsPathQueryBuilder::default()
                        .from(from)
                        .to(to)
                        .excluding_paths_via(except_via)
                        .build()?,
                )?;
                if let Some(path) = path {
                    violations.push(ContractViolation::ForbiddenInternalImport {
                        forbidden_import: forbidden_import.clone(),
                        path,
                    })
                };
                Ok(violations)
            },
        )
        .try_reduce(
            Vec::new,
            |mut all_violations, violations| -> anyhow::Result<_> {
                all_violations.extend(violations);
                Ok(all_violations)
            },
        )?;

    Ok(violations)
}

pub(super) fn find_external_import_violations(
    forbidden_imports: &[ForbiddenExternalImport],
    imports_info: &ImportsInfo,
) -> Result<Vec<ContractViolation>> {
    let violations = forbidden_imports
        .into_par_iter()
        .try_fold(
            Vec::new,
            |mut violations, forbidden_import| -> anyhow::Result<_> {
                // A contract operates in "as packages" mode, meaning
                // items are expanded to include their descendants.
                let from = forbidden_import
                    .from
                    .conv::<HashSet<PackageItemToken>>()
                    .with_descendants(imports_info.package_info());
                let to = imports_info
                    .external_imports()
                    .get_equal_to_or_descendant_imports(&forbidden_import.to);
                let except_via = forbidden_import
                    .except_via()
                    .clone()
                    .with_descendants(imports_info.package_info());

                let path = imports_info.external_imports().find_path(
                    &ExternalImportsPathQueryBuilder::default()
                        .from(from)
                        .to(to)
                        .excluding_paths_via(except_via)
                        .build()?,
                )?;
                if let Some(path) = path {
                    violations.push(ContractViolation::ForbiddenExternalImport {
                        forbidden_import: forbidden_import.clone(),
                        path,
                    })
                };
                Ok(violations)
            },
        )
        .try_reduce(
            Vec::new,
            |mut all_violations, violations| -> anyhow::Result<_> {
                all_violations.extend(violations);
                Ok(all_violations)
            },
        )?;

    Ok(violations)
}
