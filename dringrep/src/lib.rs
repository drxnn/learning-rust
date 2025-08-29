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
// use multiple threads for performance
// add flag so user can choose how many of the matches to show, example --show=10 would show only 10 matches
// add flag to put all results into a file called output.txt
*/

// use colored::Colorize;

mod types;

pub use types::{Args, Config, FileResult, Pattern, ThreadPool};

pub fn count_matches(matches: &Vec<(usize, &str)>) -> usize {
    // right now only counts number of lines matches, fix so that it counts every occurrence of pattern provided
    return matches.len();
}

trait Matcher {
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

fn process_lines<'a, M: Matcher + Sized>(
    query: &M,
    contents: &'a str,
    invert: bool,
) -> Vec<(usize, &'a str)> {
    contents
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let matched = query.matches_query(line);
            if matched ^ invert {
                Some((i + 1, line))
            } else {
                None
            }
        })
        .collect()
}
pub fn search<'a>(config: &Config, contents: &'a str) -> Vec<(usize, &'a str)> {
    // match config.parttern here
    process_lines(&config.pattern, contents, config.invert)
}

#[cfg(test)]
mod tests {

    use super::*;

    // fix texts because now I include number of line in search function
    //     #[test]
    //     fn one_result() {
    //         let query = "duct";
    //         let contents = "\
    //  Rust:
    // safe, fast, productive.
    //  Pick three.";
    //         assert_eq!(
    //             vec!["safe, fast, productive."],
    //             search(query, contents, false)
    //         )
    //     }

    //     #[test]
    //     fn case_insensitive() {
    //         let query = "rUsT";
    //         let contents = "\
    // Rust:
    // safe, fast, productive.
    // Pick three.
    // Trust me.";

    //         assert_eq!(vec!["Rust:", "Trust me."], search(query, contents, false));
    //     }

    //     #[test]

    //     fn case_sensitive() {
    //         let query = "HELLO";
    //         let contents = "\
    // HELLO FROM THE OTHER SIDE.
    // HELLO THERE buDDY
    // yeah I guess hElLo.
    // hello hello hello";

    //         assert_eq!(
    //             vec!["HELLO FROM THE OTHER SIDE.", "HELLO THERE buDDY"],
    //             search(query, contents, true)
    //         );
    //     }

    // add more tests including tests about using multiple flags at the same time
}
