use failure::{bail, Error};
use graphql_parser::parse_schema;

pub fn format(contents: &str) -> Result<String, Error> {
    let ast = parse_schema(contents)?;
    unimplemented!("format schema")
}
