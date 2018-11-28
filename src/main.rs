use failure::Error;
use graphql_parser::parse_query;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use structopt::StructOpt;

#[macro_use]
mod macros;

mod diff;
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
        /// Write the formatted output back to the file
        #[structopt(short = "w", long = "write")]
        write: bool,
        /// Write the formatted output back to the file
        #[structopt(long = "check")]
        check: bool,
    },
    /// Run a query against a GraphQL web service
    #[structopt(name = "run")]
    Run {
        /// The file containing the query to run.
        file: String,
        /// The URL to the GraphQL web service
        #[structopt(short = "h", long = "host")]
        host: String,
    },
}

fn main() {
    let opt = Opt::from_args();

    let res = match opt {
        Opt::Query { file, schema } => validate_query(file, schema),
        Opt::Schema { file } => validate_schema(file),
        Opt::Validate { file } => validate(file),
        Opt::Format { file, write, check } => format(file, write, check),
        Opt::Run { file, host } => run(file, host),
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

fn format(file_path: String, write: bool, check: bool) -> Output {
    if write && check {
        eprintln!("format cannot both check and write");
        std::process::exit(1);
    }

    let contents = read_file(&file_path)?;
    let contents = contents.trim();

    let formatted = if is_schema(&contents) {
        format::schema::format(&contents)?
    } else {
        format::query::format(&contents)?
    };

    if write {
        write_file(file_path, formatted)?;
    } else if check {
        if formatted != contents {
            print_diff(&formatted, &contents);
            std::process::exit(1);
        }
    } else {
        println!("{}", formatted);
    }

    Ok(())
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

fn write_file(file_path: String, out: String) -> Result<(), Error> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::create(file_path)?;
    file.write_all(out.as_bytes())?;
    Ok(())
}

fn print_diff(formatted: &str, contents: &str) {
    use self::diff;
    let diff = diff::make_diff(contents, formatted, formatted.len());
    diff::print_diff(diff);
}

fn run(file: String, host: String) -> Result<(), Error> {
    let mut map = HashMap::new();

    let contents = read_file(&file)?;
    parse_query(&contents)?;

    map.insert("query", contents);
    map.insert("variables", "{}".to_string());

    let client = reqwest::Client::new();
    let mut res = client.post(&host).json(&map).send()?;
    let json: Value = res.json()?;
    let pretty = serde_json::to_string_pretty(&json)?;
    println!("{}", pretty);

    Ok(())
}
