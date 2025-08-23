/*
To do in order:
1) add line numbering(print line number along each match) // done if flag is set to true
2) add count mode( print the number of matches per file) // done if flag is set to true(only counts number of
lines where pattern occurs not patterns matches -- maybe add later)
3) print just the file name that contains at least 1 match // done

4) add invert match(printing lines that do not match)//  done
5) add regex support with regex-crate
7) add highlighting support, with -h flag -> use colored crates
6) add recursive with walkdir
optional: add support for numbers (for example flags that expect numeric values (--max-count 10))
*/
use clap::Parser;
use colored::Colorize;
use regex::Regex;
use regex::RegexBuilder;
use std::env;
pub enum Pattern {
    Literal {
        text: String,
        case_insensitive: bool,
    },
    Regex(Regex),
}
use std::process;

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
#[derive(Parser)]
pub struct Args {
    query: String,
    file_path: String,
    #[arg(long = "icase")]
    ignore_case: bool,
    #[arg(short, long)]
    invert: bool,
    #[arg(short = 'E', long)]
    regex: bool,
    #[arg(short = 'c', long)]
    count: bool,
    #[arg(short, long)]
    line_number: bool,
    #[arg(short, long)]
    recursive: bool,
    #[arg(short = 'n', long)]
    file_name_if_matches: bool,
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
            file_path: args.file_path,
            ignore_case,
            invert: args.invert,
            count: args.count,
            line_number: args.line_number,
            recursive: args.recursive,
            file_name_if_matches: args.file_name_if_matches,
        }
    }
}
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
    query: &'a M,
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
pub fn search<'a>(config: &'a Config, contents: &'a str) -> Vec<(usize, &'a str)> {
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
}
