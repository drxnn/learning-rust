use clap::Parser;
use colored::Colorize;
extern crate num_cpus;
use minigrep::{Args, Config, count_matches, search};

use std::collections::HashMap;
use std::env;
use std::error::Error;

// use crossbeam::channel;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::DirEntry;
use walkdir::WalkDir;

fn main() {
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

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                // so fetching of jobs is done at one time per worker, but the processing happens concurrently, so maybe send 50-100 files per worker
                // at a time instead,
                let job = {
                    let recv_lock = receiver.lock().unwrap();
                    recv_lock.recv()
                };

                match job {
                    Ok(job) => {
                        job();
                    }
                    Err(_) => break,
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id as usize, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender.take();
        for worker in &mut self.workers {
            if let Some(t) = worker.thread.take() {
                t.join().unwrap();
            }
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        if let Some(sender) = &self.sender {
            sender.send(job).expect("Worker thread has shut down");
        } else {
            panic!("ThreadPool has been shut down");
        }
    }
}

enum FileResult {
    Match(String, Vec<(usize, String)>),
    Skip, // skip file in this case
    Error(String),
}

// run is bloated right now
// break it down into smaller functions, such as output formatting, file scanning fn, batch processing(send 50-100 files per worker)

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

            // is file valid to check or skip it
            if std::str::from_utf8(&bytes).is_err() {
                return FileResult::Skip;
            }
            let file_name = entry.file_name();

            let file_contents = String::from_utf8_lossy(&bytes);
            let temp = search(&config, &file_contents);

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

fn print_results(out_map: HashMap<String, Vec<(usize, String)>>, config: Arc<Config>) {
    for (key, value) in &out_map {
        for (i, line) in value {
            if config.line_number {
                println!("{} - line: {}, {}", key.green(), i, line);
            } else {
                println!("{}: {}", key.green(), line);
            }
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
    let current = env::current_dir().unwrap();

    let config = Arc::new(config);
    if config.recursive {
        let num_of_cpus = num_cpus::get();
        let pool_size = if num_of_cpus > 1 { num_of_cpus - 1 } else { 1 };
        let mut entry: DirEntry;
        let thread_pool = ThreadPool::new(pool_size);

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
                    // output.output_map.insert(n, v);
                    // maybe instead of inserting and then printing later, just feed files to print here

                    for (key, value) in &v {
                        let config = Arc::clone(&config);
                        print_each_result(config, &n, (*key, value));
                    }
                }
                FileResult::Error(e) => eprintln!("Error: {}", e),
                FileResult::Skip => {}
            }
        }
        // print results

        // print_results(output.output_map, config);
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
