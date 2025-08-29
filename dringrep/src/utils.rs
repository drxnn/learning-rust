use colored::Colorize;

extern crate num_cpus;

use crate::{Args, Config, FileResult, ThreadPool, search};
use std::fs;

use std::sync::Arc;

use std::sync::mpsc;

use walkdir::DirEntry;

pub fn process_batch(batch: Vec<DirEntry>, tx: mpsc::Sender<FileResult>, config: Arc<Config>) {
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

pub fn print_each_result(config: Arc<Config>, name: &str, v: (usize, &String)) {
    if config.line_number {
        println!("{} - line: {}, {}", name.green(), v.0, v.1);
    } else {
        println!("{}: {}", name.green(), v.1);
    }
}
