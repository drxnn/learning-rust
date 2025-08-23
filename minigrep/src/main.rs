use clap::Parser;
use minigrep::{Args, Config, count_matches, search};

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

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file_contents_bytes = fs::read(&config.file_path)?;
    let file_contents = String::from_utf8(file_contents_bytes)?;
    let output = search(&config, &file_contents);

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
