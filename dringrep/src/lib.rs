/*
To do in order:
1) add line numbering(print line number along each match) // done if flag is set to true
2) add count mode( print the number of matches per file) // done if flag is set to true(only counts number of
lines where pattern occurs not patterns matches -- maybe add later)
3) print just the file name that contains at least 1 match // done

4) add invert match(printing lines that do not match)//  done
5) add regex support with regex-crate done
6) add recursive with walkdir done
7) add highlighting support, with -h flag -> use colored crates half done
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

pub fn highlight_match<'a>(line: &str, pattern: &Pattern) -> String {
    let mut highlighted_string = String::from("");

    let mut matched_indices: Vec<(usize, usize)> = Vec::new();

    let pat_len = pattern.fixed_len();

    match pattern {
        Pattern::Literal {
            text,
            case_insensitive,
        } => {
            for (start_index, _char) in line.char_indices() {
                //
                for (end_index, _char) in
                    line.char_indices().skip_while(|(i, _c)| *i <= start_index)
                /*this is done because safety in regards to byte indices for certain chars */
                {
                    // doesnt account for regex, figure out
                    if end_index - start_index > pat_len.unwrap() {
                        break;
                    }

                    let sub_string = &line[start_index..end_index];

                    if pattern.matches_query(sub_string) {
                        matched_indices.push((start_index, end_index));
                    }
                }
            }

            for (index, char) in line.char_indices() {
                let inside_match = matched_indices
                    .iter()
                    .any(|(s, e)| index >= *s && index < *e);

                if inside_match {
                    highlighted_string
                        .push_str(&char.to_string().red().underline().bold().to_string());
                } else {
                    highlighted_string.push(char);
                }
            }

            highlighted_string
        }
        Pattern::Regex(re) => {
            // use captures to highlight
            // let mut out = Vec::new();
            // maybe add flag to include overlaps. Example: "ana" is twice in "banana" but the code below will only highlight the first match
            let mut pos = 0usize;
            let matches: Vec<(usize, usize)> =
                re.find_iter(line).map(|x| (x.start(), x.end())).collect();

            for (start_of, end_of) in &matches {
                println!("first item: {}, second item: {}", start_of, end_of);
            }
            for (index, char) in line.char_indices() {
                let inside_match = matches.iter().any(|(s, e)| index >= *s && index < *e);

                if inside_match {
                    highlighted_string
                        .push_str(&char.to_string().red().underline().bold().to_string());
                } else {
                    highlighted_string.push(char);
                }
            }

            highlighted_string
        }
    }
}

fn process_lines<'a>(
    query: &Pattern,
    contents: &'a str,
    invert: bool,
    highlight: bool,
) -> Vec<(usize, String)> {
    // performance is absolutely horrible below matches_query is O(n*m) and its inside the filter_map.
    // highlight_match has two nested loops and its called inside filter_map
    // change logic
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
