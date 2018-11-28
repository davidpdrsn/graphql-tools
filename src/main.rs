use failure::Error;
use graphql_parser::parse_query;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::StatusCode;
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
    /// Validate a query by running it and seeing if it works
    #[structopt(name = "validate")]
    Validate {
        /// The file to validate
        file: String,
        /// The host to send the query to
        #[structopt(name = "host", short = "h")]
        host: String,
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
    },
}

fn main() {
    let opt = Opt::from_args();

    let res = match opt {
        Opt::Validate { file, host } => validate_query(file, host),
        Opt::Schema { file } => validate_schema(file),
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

fn validate_query(query_path: String, host: String) -> Output {
    use glob::glob;

    let mut failed_files = vec![];

    glob(&query_path)?
        .into_iter()
        .filter_map(|file| file.ok())
        .map(|file| file.to_string_lossy().into_owned())
        .filter(|file| !is_schema(&read_file(file).expect("unreadable file from glob")))
        .for_each(|file| {
            let (_, status) =
                run_2(file.to_string(), host.clone()).expect("request failed to execute");
            if status.is_success() {
                println!("✅ {}", file);
            } else {
                println!("⛔️ {}", file);
                failed_files.push(file);
            }
        });

    if !failed_files.is_empty() {
        println!("\nFailures:");
        for file in failed_files {
            println!("{}", file);
        }
    }

    Ok(())
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

fn run(file: String, host: String) -> Result<(), Error> {
    let (json, status) = run_2(file, host)?;
    let pretty = serde_json::to_string_pretty(&json)?;

    println!("{}", status);
    println!("{}", pretty);

    if !status.is_success() {
        std::process::exit(1);
    }

    Ok(())
}

fn run_2(file: String, host: String) -> Result<(Value, StatusCode), Error> {
    let mut map = HashMap::new();

    let contents = read_file(&file)?;
    parse_query(&contents)?;

    map.insert("query", contents);
    map.insert("variables", "{}".to_string());

    let client = reqwest::Client::new();
    let mut res = client.post(&host).json(&map).send()?;

    let status = res.status();
    let json: Value = res.json()?;
    Ok((json, status))
}
