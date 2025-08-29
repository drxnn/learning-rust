use clap::Parser;
use colored::Colorize;
extern crate num_cpus;

mod types;

use dringrep::{Args, Config, FileResult, ThreadPool, count_matches, search};

use std::env;
use std::error::Error;

// use crossbeam::channel;
use std::fs;

use std::process;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::Instant;

use walkdir::DirEntry;
use walkdir::WalkDir;

fn main() {
    let args = Args::parse();
    let config: Config = args.into();
    let start = Instant::now();

    // dont need return value so we use if let
    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
    let duration = start.elapsed();
    println!("Finished in {:?}", duration);
}

fn process_batch(batch: Vec<DirEntry>, tx: mpsc::Sender<FileResult>, config: Arc<Config>) {
    for entry in batch {
        let res = (|| -> FileResult {
            if !entry.file_type().is_file() {
                return FileResult::Skip;
            }

            let path = entry.path().to_path_buf();
            let bytes = match fs::read(&path) {
                Ok(b) => b,
                _ => {
                    return FileResult::Skip;
                }
            };

            if std::str::from_utf8(&bytes).is_err() {
                return FileResult::Skip;
            }
            let file_name = entry.file_name();

            let file_contents = String::from_utf8_lossy(&bytes);

            let temp = search(&*config, &file_contents);

            if temp.is_empty() {
                return FileResult::Skip;
            }

            let owned_temp: Vec<(usize, String)> = temp
                .into_iter()
                .map(|(idx, s)| (idx, s.to_string()))
                .collect();

            let file_name_owned = file_name.to_string_lossy().into_owned();

            // no match

            FileResult::Match(file_name_owned, owned_temp)
        })();
        if let Err(send_err) = tx.send(res) {
            eprintln!("failed to send result back to main: {:?}", send_err);
        }
    }
}

fn print_each_result(config: Arc<Config>, name: &str, v: (usize, &String)) {
    if config.line_number {
        println!("{} - line: {}, {}", name.green(), v.0, v.1);
    } else {
        println!("{}: {}", name.green(), v.1);
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file_counter = Arc::new(Mutex::new(0));
    let current = env::current_dir().unwrap();

    let config = Arc::new(config);
    if config.recursive {
        let num_of_cpus = num_cpus::get();
        let pool_size = if num_of_cpus > 1 { num_of_cpus - 1 } else { 1 };
        let mut entry: DirEntry;
        let file_counter_clone = Arc::clone(&file_counter);
        let thread_pool = ThreadPool::new(pool_size, file_counter_clone);

        const BATCH_SIZE: usize = 25;

        let mut batch = Vec::with_capacity(BATCH_SIZE);

        let (tx, rx) = mpsc::channel::<FileResult>();

        for entry_walkdir in WalkDir::new(current) {
            entry = entry_walkdir?;
            let tx = tx.clone();
            let config = Arc::clone(&config);

            batch.push(entry);

            if batch.len() == BATCH_SIZE {
                thread_pool.execute(move || process_batch(batch, tx, config));
                batch = Vec::with_capacity(BATCH_SIZE); // reset batch
            }
        }
        // if less than 25 files, send the remaining
        if !batch.is_empty() {
            let tx = tx.clone();
            let config = Arc::clone(&config);
            thread_pool.execute(move || process_batch(batch, tx, config));
        }

        drop(tx);

        for file_response in rx {
            match file_response {
                FileResult::Match(n, v) => {
                    for (key, value) in &v {
                        let config = Arc::clone(&config);
                        print_each_result(config, &n, (*key, value));
                    }
                }
                FileResult::Error(e) => eprintln!("Error: {}", e),
                FileResult::Skip => {}
            }
        }
    } else {
        let file_contents_bytes = fs::read(&config.file_path)?;
        let file_contents = String::from_utf8_lossy(&file_contents_bytes);
        let output = search(&config, &file_contents);

        let file_name = current.file_name().unwrap().to_str().unwrap();

        for (i, line) in &output {
            let config = Arc::clone(&config);
            print_each_result(config, file_name, (*i, &line.to_string()));
        }

        if config.count {
            let count_matches = count_matches(&output);
            println!("Number of matched lines found: {count_matches:?}");
        }

        if config.file_name_if_matches && output.len() > 0 {
            println!("File name: {}", config.file_path)
        }
    }

    println!(
        "the number of processed files was: {}",
        *file_counter.lock().unwrap()
    );
    Ok(())
}
