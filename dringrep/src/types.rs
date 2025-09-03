use std::collections::HashMap;
use std::path::Path;

use clap::Parser;
use regex::{Regex, RegexBuilder};
use std::env;
use std::process;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
pub enum Pattern {
    Literal {
        pattern: Vec<String>,
        case_insensitive: bool,
    },
    Regex(Regex),
    MultipleLiteral {
        pattern: Vec<String>,

        case_insensitive: bool,
    },
}

// regex len needs some solution
pub trait PatternLen {
    fn fixed_len(&self) -> Option<usize>;
}
impl PatternLen for Pattern {
    fn fixed_len(&self) -> Option<usize> {
        match self {
            Pattern::Literal {
                pattern,
                case_insensitive,
            } => Some(pattern.first().unwrap().len()),

            _ => None,
        }
    }
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
    pub file_extension: Option<String>,
    pub highlight: bool,
}
pub struct Output {
    pub output_map: HashMap<String, Vec<(usize, String)>>,
}
#[derive(Parser)]
pub struct Args {
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long, num_args = 1.., conflicts_with = "regex")]
    pub multiple: Vec<String>,

    #[arg(long = "icase")]
    pub ignore_case: bool,

    #[arg(short = 'F', long, value_name = "FILE_PATH")]
    pub file_path: Option<String>,

    #[arg(short, long)]
    pub invert: bool,
    #[arg(short = 'E', long, conflicts_with = "multiple")]
    pub regex: bool,
    #[arg(short = 'c', long)]
    pub count: bool,
    #[arg(short, long)]
    pub line_number: bool,
    #[arg(short = 'r', long)]
    pub recursive: bool,
    #[arg(short = 'n', long)]
    pub file_name_if_matches: bool,
    #[arg(long, value_name = "EXTENSION")]
    // to use you pass cargo run -- --file-extension .rs
    pub file_extension: Option<String>,
    #[arg(long = "highlight")]
    pub highlight: bool,
}
impl From<Args> for Config {
    fn from(args: Args) -> Self {
        let ignore_case = args.ignore_case || env::var("IGNORE_CASE").is_ok();

        let file_path = match args.file_path {
            Some(fp) => fp,
            None => {
                eprintln!("Error: no file path provided.");
                std::process::exit(1);
            }
        };

        let file_extension = args.file_extension.or_else(|| {
            Path::new(&file_path)
                .extension()
                .map(|ext| ext.to_string_lossy().to_string())
        });

        let pattern = if args.regex {
            let q = if let Some(qs) = args.query.clone() {
                qs
            } else {
                args.multiple.first().cloned().unwrap_or_else(|| {
                    eprintln!("--regex requires a query string (use --query or --multiple).");
                    std::process::exit(1);
                })
            };
            match RegexBuilder::new(&q).case_insensitive(ignore_case).build() {
                Ok(re) => Pattern::Regex(re),
                Err(e) => {
                    eprintln!("Invalid regex `{}`: {}", q, e);
                    process::exit(1);
                }
            }
        } else if !args.multiple.is_empty() {
            Pattern::MultipleLiteral {
                pattern: args.multiple.clone(),
                case_insensitive: ignore_case,
            }
        } else if let Some(q) = args.query {
            Pattern::Literal {
                pattern: vec![q],
                case_insensitive: ignore_case,
            }
        } else {
            eprintln!(
                "Error: no query provided. Provide positional argument(1) for query <Q> or --multiple <Q>."
            );
            process::exit(1);
        };

        Config {
            pattern,
            file_path,
            ignore_case,
            invert: args.invert,
            count: args.count,
            line_number: args.line_number,
            recursive: args.recursive,
            file_name_if_matches: args.file_name_if_matches,
            file_extension,
            highlight: args.highlight,
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
