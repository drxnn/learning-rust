use std::collections::HashMap;

use clap::Parser;
use regex::{Regex, RegexBuilder};
use std::env;
use std::process;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
pub enum Pattern {
    Literal {
        text: String,
        case_insensitive: bool,
    },
    Regex(Regex),
}

pub struct Config {
    pub file_path: String,
    pub pattern: Pattern,
    pub ignore_case: bool,
    pub invert: bool,
    pub count: bool,
    pub line_number: bool,
    pub recursive: bool,
    pub file_name_if_matches: bool,
}
pub struct Output {
    pub output_map: HashMap<String, Vec<(usize, String)>>,
}
#[derive(Parser)]
pub struct Args {
    pub query: String,
    pub file_path: Option<String>,
    #[arg(long = "icase")]
    pub ignore_case: bool,
    #[arg(short, long)]
    pub invert: bool,
    #[arg(short = 'E', long)]
    pub regex: bool,
    #[arg(short = 'c', long)]
    pub count: bool,
    #[arg(short, long)]
    pub line_number: bool,
    #[arg(short = 'r', long)]
    pub recursive: bool,
    #[arg(short = 'n', long)]
    pub file_name_if_matches: bool,
}
impl From<Args> for Config {
    fn from(args: Args) -> Self {
        let ignore_case = args.ignore_case || env::var("IGNORE_CASE").is_ok();

        let pattern = if args.regex {
            match RegexBuilder::new(&args.query)
                .case_insensitive(ignore_case)
                .build()
            {
                Ok(re) => Pattern::Regex(re),
                Err(e) => {
                    eprintln!("Invalid regex `{}`: {}", args.query, e);
                    process::exit(1);
                }
            }
        } else {
            Pattern::Literal {
                text: args.query.clone(),
                case_insensitive: ignore_case,
            }
        };

        Config {
            pattern,
            file_path: args.file_path.unwrap_or("".to_string()),
            ignore_case,
            invert: args.invert,
            count: args.count,
            line_number: args.line_number,
            recursive: args.recursive,
            file_name_if_matches: args.file_name_if_matches,
        }
    }
}
pub struct ThreadPool {
    pub workers: Vec<Worker>,
    pub sender: Option<mpsc::Sender<Job>>,
}

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        counter: Arc<Mutex<usize>>,
    ) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let job = {
                    let recv_lock = receiver.lock().unwrap();
                    recv_lock.recv()
                };

                match job {
                    Ok(job) => {
                        job();
                        let mut count = counter.lock().unwrap();
                        *count += 1;
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
    pub fn new(size: usize, counter: Arc<Mutex<usize>>) -> Self {
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            let counter_clone = Arc::clone(&counter);
            workers.push(Worker::new(
                id as usize,
                Arc::clone(&receiver),
                counter_clone,
            ));
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

pub type Job = Box<dyn FnOnce() + Send + 'static>;

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

pub enum FileResult {
    Match(String, Vec<(usize, String)>),
    Skip,
    Error(String),
}
