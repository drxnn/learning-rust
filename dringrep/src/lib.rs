/*
To do in order:
1) add line numbering(print line number along each match) // done if flag is set to true
2) add count mode( print the number of matches per file) // done if flag is set to true(only counts number of
lines where pattern occurs not patterns matches -- maybe add later)
3) print just the file name that contains at least 1 match // done

4) add invert match(printing lines that do not match)//  done
5) add regex support with regex-crate
6) add recursive with walkdir
7) add highlighting support, with -h flag -> use colored crates
8) add option to include a file extension for files you want to check, or you dont want to check.


optional: add support for numbers (for example flags that expect numeric values (--max-count 10))

// check out aho_corasick crate for search algo

*/

mod types;
mod utils;

use colored::Colorize;
pub use types::{Args, Config, FileResult, Pattern, ThreadPool};
pub use utils::{print_each_result, print_results, process_batch};

use crate::types::PatternLen;

pub fn count_matches(matches: &Vec<(usize, String)>) -> usize {
    // wrong for recursive, fix

    return matches.len();
}

pub trait Matcher {
    fn matches_query(&self, text: &str) -> bool;
}

impl Matcher for Pattern {
    fn matches_query(&self, text: &str) -> bool {
        match self {
            Pattern::Literal {
                text: pattern,
                case_insensitive,
            } => {
                if *case_insensitive {
                    text.to_lowercase().contains(&pattern.to_lowercase())
                } else {
                    text.contains(pattern)
                }
            }
            Pattern::Regex(re) => re.is_match(text),
        }
    }
}

pub fn highlight_match<'a, T: Matcher + PatternLen>(line: &str, pattern: &T) -> String {
    let mut highlighted_string = String::from("");

    let mut matched_indices: Vec<(usize, usize)> = Vec::new();

    let pat_len = pattern.fixed_len();

    for (start_index, _char) in line.char_indices() {
        //
        for (end_index, _char) in line.char_indices().skip_while(|(i, c)| *i <= start_index)
        /*this is done because safety in regards to byte indices for certain chars */
        {
            // doesnt account for regex, figure out
            if end_index - start_index > pat_len.unwrap() {
                break;
            }
            //hee hee he hehehehhee

            let sub_string = &line[start_index..end_index];

            if pattern.matches_query(sub_string) {
                matched_indices.push((start_index, end_index));
            }
        }
    }

    println!("line is: {}", line);

    for (index, char) in line.char_indices() {
        {
            let inside_match = matched_indices
                .iter()
                .any(|(s, e)| index >= *s && index < *e);

            if inside_match {
                highlighted_string.push_str(&char.to_string().red().underline().bold().to_string());
            } else {
                highlighted_string.push(char);
            }
        }
    }

    highlighted_string
}

fn process_lines<'a, M: Matcher + Sized + PatternLen>(
    query: &M,
    contents: &'a str,
    invert: bool,
    highlight: bool,
) -> Vec<(usize, String)> {
    contents
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let matched = query.matches_query(line);
            if matched ^ invert {
                if highlight {
                    return Some((i + 1, highlight_match(line, query)));
                } else {
                    return Some((i + 1, line.to_string()));
                }
            } else {
                None
            }
        })
        .collect::<Vec<(usize, String)>>()
}
pub fn search<'a>(config: &Config, contents: &'a str) -> Vec<(usize, String)> {
    // match config.parttern here
    process_lines(&config.pattern, contents, config.invert, config.highlight)
}

#[cfg(test)]
mod tests {
    // test all flags work correctly
    // text various regex patterns
    // text error handling
}
