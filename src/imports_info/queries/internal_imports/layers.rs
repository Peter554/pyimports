#![allow(dead_code)] // TODO: Remove me

use crate::PackageItemToken;
use maplit::hashset;
use std::collections::HashSet;

#[derive(Debug, Clone)]
struct Layer {
    siblings: HashSet<PackageItemToken>,
    siblings_independent: bool,
}

impl Layer {
    fn new<T: IntoIterator<Item = PackageItemToken>>(
        siblings: T,
        siblings_independent: bool,
    ) -> Self {
        Layer {
            siblings: siblings.into_iter().collect(),
            siblings_independent,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ForbiddenImport {
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
}

fn get_forbidden_imports(layers: &[Layer]) -> Vec<ForbiddenImport> {
    let mut forbidden_imports = Vec::new();

    for (idx, layer) in layers.iter().enumerate() {
        for higher_layer in layers[idx + 1..].iter() {
            for layer_sibling in layer.siblings.iter() {
                for higher_layer_sibling in higher_layer.siblings.iter() {
                    forbidden_imports.push(ForbiddenImport::new(
                        *layer_sibling,
                        *higher_layer_sibling,
                        hashset! {},
                    ));
                }
            }
        }

        if idx >= 2 {
            let directly_lower_layer = &layers[idx - 1];
            for lower_layer in layers[..idx - 1].iter() {
                for layer_sibling in layer.siblings.iter() {
                    for lower_layer_sibling in lower_layer.siblings.iter() {
                        forbidden_imports.push(ForbiddenImport::new(
                            *layer_sibling,
                            *lower_layer_sibling,
                            directly_lower_layer.siblings.clone(),
                        ));
                    }
                }
            }
        }

        if layer.siblings_independent {
            for layer_sibling1 in layer.siblings.iter() {
                for layer_sibling2 in layer.siblings.iter() {
                    if layer_sibling1 == layer_sibling2 {
                        continue;
                    }
                    forbidden_imports.push(ForbiddenImport::new(
                        *layer_sibling1,
                        *layer_sibling2,
                        hashset! {},
                    ));
                }
            }
        }
    }

    forbidden_imports
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PackageToken;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use slotmap::SlotMap;

    #[test]
    fn test_get_forbidden_imports() -> Result<()> {
        let mut sm: SlotMap<PackageToken, String> = SlotMap::with_key();
        let data: PackageItemToken = sm.insert("data".into()).into();
        let domain1: PackageItemToken = sm.insert("domain1".into()).into();
        let domain2: PackageItemToken = sm.insert("domain2".into()).into();
        let application1: PackageItemToken = sm.insert("application1".into()).into();
        let application2: PackageItemToken = sm.insert("application2".into()).into();
        let interfaces: PackageItemToken = sm.insert("interfaces".into()).into();

        let layers = vec![
            Layer::new([data], true),
            Layer::new([domain1, domain2], true),
            Layer::new([application1, application2], false),
            Layer::new([interfaces], true),
        ];

        let forbidden_imports = get_forbidden_imports(&layers);

        let expected = vec![
            // data may not import domain, application or interfaces
            ForbiddenImport::new(data, domain1, hashset! {}),
            ForbiddenImport::new(data, domain2, hashset! {}),
            ForbiddenImport::new(data, application1, hashset! {}),
            ForbiddenImport::new(data, application2, hashset! {}),
            ForbiddenImport::new(data, interfaces, hashset! {}),
            // domain may not import application or interfaces
            // (domain may import data)
            ForbiddenImport::new(domain1, application1, hashset! {}),
            ForbiddenImport::new(domain1, application2, hashset! {}),
            ForbiddenImport::new(domain1, interfaces, hashset! {}),
            ForbiddenImport::new(domain2, application1, hashset! {}),
            ForbiddenImport::new(domain2, application2, hashset! {}),
            ForbiddenImport::new(domain2, interfaces, hashset! {}),
            // domain1 and domain2 are independent siblings
            ForbiddenImport::new(domain1, domain2, hashset! {}),
            ForbiddenImport::new(domain2, domain1, hashset! {}),
            // application may not import interfaces
            // application may not import data, except via domain
            // (application may import domain)
            ForbiddenImport::new(application1, interfaces, hashset! {}),
            ForbiddenImport::new(application1, data, hashset! {domain1, domain2}),
            ForbiddenImport::new(application2, interfaces, hashset! {}),
            ForbiddenImport::new(application2, data, hashset! {domain1, domain2}),
            // interfaces may not import data or domain, except via application
            // (application may import application)
            ForbiddenImport::new(interfaces, data, hashset! {application1, application2}),
            ForbiddenImport::new(interfaces, domain1, hashset! {application1, application2}),
            ForbiddenImport::new(interfaces, domain2, hashset! {application1, application2}),
        ];

        assert_eq!(forbidden_imports.len(), expected.len(),);
        for forbidden_import in forbidden_imports.iter() {
            assert!(expected.contains(forbidden_import));
        }

        Ok(())
    }
}
