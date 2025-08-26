use clap::Parser;
use colored::Colorize;
use minigrep::{Args, Config, count_matches, search};

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::process;
use walkdir::DirEntry;
use walkdir::WalkDir;

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
struct Output {
    output_map: HashMap<String, Vec<(usize, String)>>,
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let current = env::current_dir().unwrap();
    let mut output = Output {
        output_map: HashMap::new(),
    };

    if config.recursive {
        let mut entry: DirEntry;

        for entry_walkdir in WalkDir::new(current) {
            entry = entry_walkdir?;
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();

                let file_contents_bytes = fs::read(&path)?;
                // is file valid to check or skip it
                if std::str::from_utf8(&file_contents_bytes).is_err() {
                    continue;
                }
                let file_name = entry.file_name();
                let file_contents_bytes = fs::read(path)?;

                let file_contents = String::from_utf8_lossy(&file_contents_bytes);
                let temp = search(&config, &file_contents);

                let owned_temp: Vec<(usize, String)> = temp
                    .into_iter()
                    .map(|(idx, s)| (idx, s.to_string()))
                    .collect();

                let file_name_owned = file_name.to_string_lossy().into_owned();

                // no match
                if owned_temp.len() > 0 {
                    output.output_map.insert(file_name_owned, owned_temp);
                } else {
                    continue;
                }
            }
        }

        if output.output_map.len() == 0 {
            eprintln!("{}", "No files match your pattern.".red());
            return Ok(());
        }

        // use iterator later
        for (key, value) in &output.output_map {
            for (i, line) in value {
                if config.line_number {
                    println!("{} - line: {}, {}", key.green(), i, line);
                } else {
                    println!("{}: {}", key.green(), line);
                }
            }
        }
    } else {
        let file_contents_bytes = fs::read(&config.file_path)?;
        let file_contents = String::from_utf8_lossy(&file_contents_bytes);
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
    }

    Ok(())
}
