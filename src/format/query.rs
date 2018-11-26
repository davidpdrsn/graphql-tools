use failure::{bail, Error};
use graphql_parser::parse_query;

pub fn format(contents: &str) -> Result<String, Error> {
    let ast = parse_query(contents)?;
    unimplemented!("format query")
}
