use clap::Parser;

extern crate num_cpus;

mod types;
mod utils;

use dringrep::{
    Args, Config, FileResult, ThreadPool, count_matches, print_each_result, process_batch, search,
};

use std::env;
use std::error::Error;

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
        // should be handled by processing function
        let file_contents_bytes = fs::read(&config.file_path)?;
        let file_contents = String::from_utf8_lossy(&file_contents_bytes);
        let output = search(&config, &file_contents);

        // no need for this below, can just print 1
        let file_counter_clone = Arc::clone(&file_counter);
        let mut count = file_counter_clone.lock().unwrap();
        *count += 1;

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
