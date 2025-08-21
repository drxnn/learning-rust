use clap::Parser;
use minigrep::search;
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
        }
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file_contents_bytes = fs::read(config.file_path)?;
    let file_contents = String::from_utf8(file_contents_bytes)?;

    for (i, line) in search(&config.query, &file_contents, config.case_sensitive) {
        if config.line_number {
            println!("{}: {}", i, line);
        } else {
            println!("{}", line);
        }
    }
    // println!("the file contents are: {file_contents}");
    Ok(())
}
