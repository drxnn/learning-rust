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

// check out aho_corasick crate for search algo for multiple string literals

*/

mod types;
mod utils;

use std::borrow::Cow;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use colored::Colorize;
pub use types::{Args, Config, FileResult, Pattern, ThreadPool};
pub use utils::{print_each_result, print_results, process_batch};

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
            Pattern::Regex(re) => re.is_match(text),
            Pattern::Literal {
                pattern,
                case_insensitive,
            } => {
                if *case_insensitive {
                    text.to_lowercase()
                        .contains(&pattern.first().unwrap().to_lowercase())
                } else {
                    text.contains(pattern.first().unwrap())
                }
            }
            // check if correct later
            Pattern::MultipleLiteral {
                pattern,
                case_insensitive,
            } => {
                let lowered_patterns: Vec<String> =
                    pattern.iter().map(|s| s.to_lowercase()).collect();
                let pattern_refs: Vec<&str> = lowered_patterns.iter().map(|s| s.as_str()).collect();

                let ac = if *case_insensitive {
                    AhoCorasickBuilder::new()
                        .build(&pattern_refs)
                        .expect("error")
                } else {
                    let pattern_refs: Vec<&str> = pattern.iter().map(|s| s.as_str()).collect();
                    AhoCorasick::new(&pattern_refs).expect("error")
                };
                ac.is_match(text)
            }
        }
    }
}

pub fn highlight_match(line: &str, pat: &Pattern) -> String {
    let mut highlighted_string = String::from("");

    match pat {
        Pattern::Literal {
            pattern,
            case_insensitive,
        }
        | Pattern::MultipleLiteral {
            pattern,
            case_insensitive,
        } => {
            let pattern_refs: Vec<&str> = pattern.iter().map(|s| s.as_str()).collect();
            let ac = if *case_insensitive {
                AhoCorasickBuilder::new()
                    .ascii_case_insensitive(true)
                    .build(&pattern_refs)
                    .unwrap()
            } else {
                AhoCorasick::new(&pattern_refs).unwrap()
            };

            let matches: Vec<(usize, usize)> =
                ac.find_iter(line).map(|m| (m.start(), m.end())).collect();

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
        Pattern::Regex(re) => {
            let matches: Vec<(usize, usize)> =
                re.find_iter(line).map(|x| (x.start(), x.end())).collect();

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
) -> Vec<(usize, Cow<'a, str>)> {
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
                    return Some((i + 1, Cow::Owned(highlight_match(line, query))));
                } else {
                    return Some((i + 1, Cow::Borrowed(line)));
                }
            } else {
                None
            }
        })
        .collect()
}
pub fn search<'a>(config: &Config, contents: &'a str) -> Vec<(usize, Cow<'a, str>)> {
    process_lines(&config.pattern, contents, config.invert, config.highlight)
}

#[cfg(test)]
mod tests {
    // test all flags work correctly
    // text various regex patterns
    // text error handling
}
