use colored::Colorize;

extern crate num_cpus;

use crate::count_matches;
use crate::{Args, Config, FileResult, ThreadPool, search};
use std::fs;

use std::sync::Arc;

use std::sync::mpsc;

use walkdir::DirEntry;

pub fn print_results(rx: mpsc::Receiver<FileResult>, config: Arc<Config>) {
    eprintln!("print_results START");
    for file_response in rx {
        match file_response {
            FileResult::Match(n, v) => {
                let config = Arc::clone(&config);
                if config.count {
                    let count_matches = count_matches(&v);
                    println!("Number of matched lines found: {count_matches:?}");
                }

                if config.file_name_if_matches && v.len() > 0 {
                    println!("File name: {}", config.file_path)
                }
                for (key, value) in &v {
                    let config = Arc::clone(&config);
                    print_each_result(config, &n, (*key, value));
                }
            }
            FileResult::Error(e) => eprintln!("Error: {}", e),
            FileResult::Skip => {}
        }
    }
    eprintln!("print_results END");
}

pub fn process_batch(batch: Vec<DirEntry>, tx: mpsc::Sender<FileResult>, config: Arc<Config>) {
    for entry in batch {
        eprintln!("process_batch START");
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
        eprintln!("process_batch END");
    }
}

pub fn print_each_result(config: Arc<Config>, name: &str, v: (usize, &String)) {
    if config.line_number {
        println!("{} - line: {}, {}", name.green(), v.0, v.1);
    } else {
        println!("{}: {}", name.green(), v.1);
    }
}
