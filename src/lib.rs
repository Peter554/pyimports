#[cfg(test)]
mod testutils;

use anyhow::Result;

fn foo() -> Result<u8> {
    Ok(42)
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use testutils::TestPackage;

    #[test]
    fn test_foo() -> Result<()> {
        let test_package = TestPackage::new(hashmap! {
            "__init__" => "",
            "a" => "",
            "b" => "",
        })?;

        println!("{:?}", test_package.path());

        assert_eq!(foo().unwrap(), 42);
        Ok(())
    }
}
