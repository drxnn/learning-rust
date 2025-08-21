use clap::Parser;
use minigrep::{count_matches, search};

use std::env;
use std::error::Error;
use std::fs;
use std::process;
fn main() {
    // args could be empty, fix
    let args = Args::parse();
    let config: Config = args.into();

    // dont need return value so we use if let
    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

struct Config {
    query: String,
    file_path: String,
    case_sensitive: bool,
    invert: bool,
    count: bool,
    line_number: bool,
    recursive: bool,
    file_name_if_matches: bool,
}
#[derive(Parser)]
struct Args {
    query: String,
    file_path: String,
    #[arg(long = "ic")]
    ignore_case: bool,
    #[arg(short, long)]
    invert: bool,
    #[arg(short, long)]
    count: bool,
    #[arg(short, long)]
    line_number: bool,
    #[arg(short, long)]
    recursive: bool,
    #[arg(short = 'n', long)]
    file_name_if_matches: bool,
}
impl From<Args> for Config {
    fn from(args: Args) -> Self {
        let case_sensitive = !args.ignore_case && env::var("IGNORE_CASE").is_err();
        Config {
            query: args.query,
            file_path: args.file_path,
            case_sensitive,
            invert: args.invert,
            count: args.count,
            line_number: args.line_number,
            recursive: args.recursive,
            file_name_if_matches: args.file_name_if_matches,
        }
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file_contents_bytes = fs::read(&config.file_path)?;
    let file_contents = String::from_utf8(file_contents_bytes)?;
    let output = search(&config.query, &file_contents, config.case_sensitive);

    for (i, line) in &output {
        if config.line_number {
            println!("{}: {}", i, line);
        } else {
            println!("{}", line);
        }
    }

    if config.count {
        let count_matches = count_matches(&output);
        println!("Number of matched lines found: {count_matches:?}");
    }

    if config.file_name_if_matches && output.len() > 0 {
        println!("File name: {}", config.file_path)
    }
    // println!("the file contents are: {file_contents}");
    Ok(())
}
