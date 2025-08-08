use minigrep::search;
use std::env;
use std::error::Error;
use std::fs;
use std::process;

fn main() {
    // args could be empty, fix
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1)
    });

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
}

impl Config {
    pub fn new(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();
        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("did not get a query string"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("did not get a file_path string"),
        };

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            case_sensitive: !ignore_case,
        })
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file_contents_bytes = fs::read(config.file_path)?;
    let file_contents = String::from_utf8(file_contents_bytes)?;

    for line in search(&config.query, &file_contents, config.case_sensitive) {
        println!("{line}")
    }
    // println!("the file contents are: {file_contents}");
    Ok(())
}

// TODO
// add regex support
// add recursive directory traversal -- check walkdir crate
// add flags
// add support for numbers
// invert match(lines that dont match the pattern)
// count  only flag -count that counts the number of  matching lines per file
// list files that have lines taht match
