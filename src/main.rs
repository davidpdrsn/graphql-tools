use failure::{bail, Error};
use lazy_static::lazy_static;
use regex::Regex;
use structopt::StructOpt;

#[macro_use]
mod macros;

mod format;

#[derive(StructOpt, Debug)]
#[structopt(name = "gqltools", about = "GraphQL tools")]
enum Opt {
    /// Validate a query against a schema
    #[structopt(name = "query")]
    Query {
        /// The file to validate
        file: String,
        /// The file to validate
        #[structopt(name = "schema", short = "s")]
        schema: String,
    },
    /// Validate a schema for internal consistency
    #[structopt(name = "schema")]
    Schema {
        /// The file to validate
        file: String,
    },
    /// Validate a query or a schema
    #[structopt(name = "validate")]
    Validate {
        /// The file to validate. The fil will only be validated for syntax errors.
        /// Use other subcommands for more specific validations.
        /// It'll be inferred from the contents if its a query or a schema.
        file: String,
    },
    /// Format a query or a schema
    #[structopt(name = "format")]
    Format {
        /// The file to format.
        /// It'll be inferred from the contents if its a query or a schema.
        file: String,
    },
}

fn main() {
    let opt = Opt::from_args();

    let res = match opt {
        Opt::Query { file, schema } => validate_query(file, schema),
        Opt::Schema { file } => validate_schema(file),
        Opt::Validate { file } => validate(file),
        Opt::Format { file } => format(file),
    };

    match res {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
}

type Output = Result<(), Error>;

fn validate_query(query_path: String, schema_path: String) -> Output {
    unimplemented!()
}

fn validate_schema(schema_path: String) -> Output {
    unimplemented!()
}

fn validate(file_path: String) -> Output {
    unimplemented!()
}

fn format(file_path: String) -> Output {
    let contents = read_file(&file_path)?;

    if is_query(&contents) {
        format::query::format(&contents)?;
    } else if is_schema(&contents) {
        format::schema::format(&contents)?;
    } else {
        bail!("Thats neither a query nor a schema");
    }

    Ok(())
}

fn is_query(contents: &str) -> bool {
    lazy_static! {
        static ref query_re: Regex = Regex::new(r"^(query|mutation)").unwrap();
    }
    contents.lines().any(|line| query_re.is_match(line))
}

fn is_schema(contents: &str) -> bool {
    lazy_static! {
        static ref schema_re: Regex = Regex::new(r"^schema").unwrap();
    }
    contents.lines().any(|line| schema_re.is_match(line))
}

fn read_file(file: &str) -> Result<String, Error> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut f = File::open(file)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}
