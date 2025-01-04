use crate::{ImportsInfo, PackageItemToken};
use anyhow::Result;
use std::collections::HashSet;

pub mod layers;

pub trait Contract {
    fn find_violations(&self, imports_info: &ImportsInfo) -> Result<Vec<ContractViolation>>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractViolation {
    ForbiddenImport {
        forbidden_import: ForbiddenImport,
        path: Vec<PackageItemToken>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForbiddenImport {
    from: PackageItemToken,
    to: PackageItemToken,
    except_via: HashSet<PackageItemToken>,
}

impl ForbiddenImport {
    fn new(
        from: PackageItemToken,
        to: PackageItemToken,
        except_via: HashSet<PackageItemToken>,
    ) -> Self {
        ForbiddenImport {
            from,
            to,
            except_via,
        }
    }

    pub fn from(&self) -> PackageItemToken {
        self.from
    }
    pub fn to(&self) -> PackageItemToken {
        self.to
    }
    pub fn except_via(&self) -> &HashSet<PackageItemToken> {
        &self.except_via
    }
}
