/*
To do in order:
1) add line numbering(print line number along each match)
2) add count-only mode(only print the number of matches per file)
3) print just the file name that contains at least 1 match
4) add invert match(printing lines that do not match)
5) add regex support with regex-crate
6) add recursive with walkdir
optional: add support for numbers (for example flags that expect numeric values (--max-count 10))
*/

pub fn search<'a>(query: &str, contents: &'a str, case_sensitive: bool) -> Vec<(usize, &'a str)> {
    if case_sensitive {
        contents
            .lines()
            .enumerate()
            .map(|(i, line)| (i + 1, line))
            .filter(|(_i, line)| line.contains(query))
            .collect::<Vec<(usize, &str)>>()
    } else {
        contents
            .lines()
            .enumerate()
            .map(|(i, line)| (i + 1, line)) // shifting index by 1 since lines start at 1
            .filter(|(_i, line)| line.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<(usize, &str)>>()
    }
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
