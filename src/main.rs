use failure::{bail, Error};
use structopt::StructOpt;

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

fn validate_query(query_path: String, schema_path: String) -> Result<(), Error> {
    unimplemented!()
}

fn validate_schema(schema_path: String) -> Result<(), Error> {
    unimplemented!()
}

fn validate(file_path: String) -> Result<(), Error> {
    unimplemented!()
}

fn format(file_path: String) -> Result<(), Error> {
    unimplemented!()
}
