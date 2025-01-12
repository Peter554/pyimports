use crate::contracts::{ContractViolation, ForbiddenInternalImport};
use crate::imports_info::{ImportsInfo, InternalImportsPathQueryBuilder};
use crate::package_info::PackageItemToken;
use crate::prelude::*;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashSet;
use tap::prelude::*;

pub(super) fn find_violations(
    forbidden_imports: Vec<ForbiddenInternalImport>,
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
                        forbidden_import,
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
