use super::Indentation;
use failure::{bail, Error};
use graphql_parser::{parse_query, query::*};

type Output = Result<(), Error>;

pub fn format(contents: &str) -> Result<String, Error> {
    let ast = parse_query(contents)?;

    let mut out = String::new();
    let mut indent = Indentation::new(2);
    format_doc(ast, &mut indent, &mut out)?;

    Ok(out.trim().to_string())
}

fn format_doc(doc: Document, indent: &mut Indentation, out: &mut String) -> Output {
    for def in doc.definitions {
        format_def(def, indent, out)?;
    }
    Ok(())
}

fn format_def(def: Definition, indent: &mut Indentation, out: &mut String) -> Output {
    match def {
        Definition::Operation(operation) => todo!("Operation"),
        Definition::Fragment(fragment) => todo!("Fragment"),
    }
    Ok(())
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_something() {
        let query = "
query Name { id }
        "
        .trim();

        assert_eq!(
            format(query).unwrap(),
            "
query Name {
  id
}
            "
            .trim(),
        );
    }
}
