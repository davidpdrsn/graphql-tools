use failure::Error;
use graphql_parser::parse_query;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, USER_AGENT},
    StatusCode,
};
use serde_json::{json, map::Map, Value};
use std::collections::HashMap;
use structopt::StructOpt;

#[macro_use]
mod macros;

mod diff;
mod format;

macro_rules! unwrap_or_exit {
    ( $e:expr, $msg:expr ) => {
        match $e {
            Ok(value) => value,
            Err(err) => {
                eprintln!($msg);
                eprintln!("{:?}", err);
                std::process::exit(1)
            }
        }
    };
}

#[derive(StructOpt, Debug)]
#[structopt(name = "gqltools", about = "GraphQL tools")]
enum Opt {
    /// Validate a query by running it and seeing if it works
    #[structopt(name = "validate")]
    Validate {
        /// File path to the query to validate
        ///
        /// Supports glob patterns such as "queries/**/*.graphql"
        #[structopt(short = "q", long = "query")]
        query: String,
        /// File path to the schema
        #[structopt(short = "s", long = "schema")]
        schema: String,
    },
    /// Validate a schema for internal consistency
    #[structopt(name = "schema")]
    Schema {
        /// The file to validate
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
        /// Add headers to the request
        ///
        /// Should be string of the form
        ///   -H "Name: Value" -H "Other-Nmae = Other Value"
        #[structopt(short = "H", long = "header")]
        headers: Vec<String>,
        /// Variables to send along with the query
        ///
        /// Should be string of the form
        ///   -v "someVarName = 1" -v "someOtherVarName = \"foo\""
        #[structopt(short = "v", long = "var")]
        vars: Vec<String>,
    },
}

fn main() {
    let opt = Opt::from_args();

    let res = match opt {
        Opt::Validate { query, schema } => validate_query(query, schema),
        Opt::Schema { file } => validate_schema(file),
        Opt::Format { file, write, check } => format(file, write, check),
        Opt::Run {
            file,
            host,
            headers,
            vars,
        } => run(file, host, headers, vars),
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
    use glob::glob;
    use colored::*;

    let mut all_good = true;
    let mut lines = Vec::new();

    glob(&query_path)?
        .filter_map(|file| file.ok())
        .map(|file| file.to_string_lossy().into_owned())
        .filter(|file| !is_schema(&read_file(file).expect("unreadable file from glob")))
        .for_each(|file| {
            match perform_validation(&file, &schema_path) {
                Ok(()) => {
                    lines.push(format!("{} {}", "OK:".green(), file));
                },
                Err(err) => {
                    lines.push(format!("{} {}", "Error:".red(), file));
                    lines.push(format!("{} {}", "Error message:".red(), err.to_string()));
                    all_good = false;
                }
            }
        });

    if !lines.is_empty() {
        println!("{}", lines.join("\n"));
    }

    if !all_good {
        std::process::exit(1);
    }

    Ok(())
}

fn perform_validation(query_path: &str, schema_path: &str) -> Result<(), Error> {
    use graphql_client_codegen::{generate_module_token_stream, GraphQLClientDeriveOptions};
    use std::path::{Path, PathBuf};
    use syn::Visibility;

    let query_path = PathBuf::from(query_path);
    let schema_path = Path::new(schema_path);
    let options = GraphQLClientDeriveOptions {
        operation_name: Some("OperationName".into()),
        struct_name: Some("StructName".into()),
        module_name: Some("module_name".into()),
        additional_derives: None,
        deprecation_strategy: None,
        module_visibility: Visibility::Inherited,
    };
    generate_module_token_stream(query_path, schema_path, Some(options)).map(|_| ())
}

fn validate_schema(_: String) -> Output {
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

fn run(file: String, host: String, headers: Vec<String>, vars: Vec<String>) -> Result<(), Error> {
    let (json, status) = run_2(file, host, headers, vars)?;
    let pretty = colored_json::to_colored_json_auto(&json)?;

    println!("{}", status);
    println!("{}", pretty);

    if !status.is_success() {
        std::process::exit(1);
    }

    Ok(())
}

fn run_2(
    file: String,
    host: String,
    headers: Vec<String>,
    vars: Vec<String>,
) -> Result<(Value, StatusCode), Error> {
    let contents = read_file(&file)?;
    parse_query(&contents)?;

    let mut map = Map::new();
    map.insert("query".to_string(), json!(contents));
    let vars = parse_variables(vars);
    map.insert("variables".to_string(), vars);

    let client = reqwest::Client::new();

    let mut res = client
        .post(&host)
        .headers(parse_headers(headers))
        .json(&map)
        .send()?;

    let status = res.status();

    if status == 200 {
        let json: Value = res.json()?;
        Ok((json, status))
    } else {
        eprintln!("Error! Response status was {}", status);

        let body = res.text()?;
        eprintln!("Body:");
        let json = serde_json::from_str::<Value>(&body);
        match json {
            Ok(json) => {
                eprintln!("{}", colored_json::to_colored_json_auto(&json).unwrap());
            }
            Err(_) => {
                eprintln!("{}", body);
            }
        }

        std::process::exit(1);
    }
}

fn parse_headers(headers: Vec<String>) -> HeaderMap {
    let mut map = HeaderMap::new();

    for input in headers {
        let split = input.split(':').map(|part| part.trim()).collect::<Vec<_>>();
        if split.len() != 2 {
            eprintln!("Error parsing header");
            std::process::exit(1);
        }

        let key = unwrap_or_exit!(
            HeaderName::from_lowercase(split[0].to_lowercase().as_bytes()),
            "Invalid header name"
        );
        let value = unwrap_or_exit!(HeaderValue::from_str(split[1]), "Invalid header value");

        map.insert(key, value);
    }

    map
}

fn parse_variables(vars: Vec<String>) -> Value {
    let mut acc = Map::new();

    for var in vars {
        let split = var.split(" = ").map(|part| part.trim()).collect::<Vec<_>>();
        if split.len() != 2 {
            eprintln!("Error parsing variable");
            std::process::exit(1);
        }

        let key = split[0];
        let value = split[1];
        let value: Value = serde_json::from_str(value).unwrap();
        acc.insert(key.to_string(), value);
    }

    Value::Object(acc)
}
