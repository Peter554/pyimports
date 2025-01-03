use std::collections::HashSet;
use maplit::hashset;
use crate::{PackageItemToken, PackageItemTokenSet};

#[derive(Debug, Clone)]
struct Layer {
    siblings: HashSet<PackageItemToken>,
    siblings_independent: bool,
}

impl Layer {
    fn new<T: IntoIterator<Item=PackageItemToken>>(siblings: T, siblings_independent: bool) -> Self {
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
    except_via: PackageItemTokenSet,
}

impl ForbiddenImport {
    fn new(from: PackageItemToken, to: PackageItemToken, except_via: PackageItemTokenSet) -> Self {
        ForbiddenImport {from, to, except_via}
    }
}

fn get_forbidden_imports(layers: &[Layer]) -> Vec<ForbiddenImport> {
    let mut forbidden_imports = Vec::new();

    for (idx, layer) in layers.iter().enumerate() {
        for higher_layer in layers[idx+1..].iter() {
            for layer_sibling in layer.siblings.iter() {
                for higher_layer_sibling in higher_layer.siblings.iter() {
                    forbidden_imports.push(ForbiddenImport::new(*layer_sibling, *higher_layer_sibling, hashset! {}));
                }
            }
        }
        if idx >= 2 {
            let directly_lower_layer = &layers[idx-1];
            for lower_layer in layers[..idx-1].iter() {
                for layer_sibling in layer.siblings.iter() {
                    for lower_layer_sibling in lower_layer.siblings.iter() {
                        forbidden_imports.push(ForbiddenImport::new(*layer_sibling, *lower_layer_sibling, directly_lower_layer.siblings.clone()));
                    }
                }
            }
        }
    }

    forbidden_imports
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use slotmap::SlotMap;
    use crate::PackageToken;

    // TODO: Add siblings, both independent and not.
    #[test]
    fn test_get_forbidden_imports() -> Result<()> {
        let mut sm: SlotMap<PackageToken, String> = SlotMap::with_key();
        let data: PackageItemToken = sm.insert("data".into()).into();
        let domain: PackageItemToken = sm.insert("domain".into()).into();
        let application: PackageItemToken = sm.insert("application".into()).into();
        let interfaces: PackageItemToken = sm.insert("interfaces".into()).into();

        let layers = vec![
            Layer::new(
                [data],
                true
            ),
            Layer::new(
                [domain],
                true
            ),
            Layer::new(
                [application],
                true
            ),
            Layer::new(
                [interfaces],
                true
            ),
        ];

        let forbidden_imports = get_forbidden_imports(&layers);

        assert_eq!(
            forbidden_imports,
            vec![
                // data may not import domain, application or interfaces
                ForbiddenImport::new(data, domain, hashset! {}),
                ForbiddenImport::new(data, application, hashset! {}),
                ForbiddenImport::new(data, interfaces, hashset! {}),
                // domain may not import application or interfaces
                // (domain may import data)
                ForbiddenImport::new(domain, application, hashset! {}),
                ForbiddenImport::new(domain, interfaces, hashset! {}),
                // application may not import interfaces
                // application may not import data, except via domain
                // (application may import domain)
                ForbiddenImport::new(application, interfaces, hashset! {}),
                ForbiddenImport::new(application, data, hashset! {domain}),
                // interfaces may not import data or domain, except via application
                // (application may import application)
                ForbiddenImport::new(interfaces, data, hashset! {application}),
                ForbiddenImport::new(interfaces, domain, hashset! {application}),
            ]
        );

        Ok(())
    }
}