use clap::Parser;
use colored::Colorize;
extern crate num_cpus;
use minigrep::{Args, Config, count_matches, search};

// use core::num;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::process;
use std::sync::mpsc;
// use std::sync::{Arc, Mutex};
use std::thread;
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

pub struct ThreadPool {
    workers: Vec<Worker>,
    receiver: mpsc::Receiver<Job>,
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, sender: mpsc::Sender<usize>) -> Self {
        let thread = thread::spawn(move || {
            println!("printing from thread number {}", id);

            sender.send(id).unwrap()
        });

        Worker { id, thread }
    }
}
impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let mut workers = Vec::with_capacity(size);

        let (worker_sender, thread_receiver) = mpsc::channel();

        for id in 0..size {
            let sender: mpsc::Sender<_> = worker_sender.clone();
            workers.push(Worker::new(id as usize, sender));
        }

        let (sender, receiver) = mpsc::channel();
        ThreadPool { workers, receiver }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn execute(self) {
        // let job = Box::new(f);

        // might not work for now, receive after threads are done
        self.receiver.recv();

        for handle in self.workers {
            handle.thread.join().unwrap();
        }
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let current = env::current_dir().unwrap();
    let mut output = Output {
        output_map: HashMap::new(),
    };

    if config.recursive {
        let num_of_cpus = num_cpus::get();
        let mut entry: DirEntry;

        // but use num_cpus - 1
        let thread = ThreadPool::new(num_of_cpus);
        thread.execute();

        // how do I give files to the workers, in real time, while walking the directory

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
        println!("num of CPU's is {}", num_of_cpus);
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
