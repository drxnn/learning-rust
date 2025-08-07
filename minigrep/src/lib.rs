pub fn search<'a>(query: &str, contents: &'a str, case_sensitive: bool) -> Vec<&'a str> {
    let mut results: Vec<&str> = Vec::new();

    if case_sensitive {
        for line in contents.lines() {
            if line.contains(query) {
                results.push(line)
            }
        }
    } else {
        let query_lower = query.to_lowercase();
        for line in contents.lines() {
            if line.to_lowercase().contains(&query_lower) {
                results.push(line);
            }
        }
    }

    results
}

// pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
 Rust:
safe, fast, productive.
 Pick three.";
        assert_eq!(
            vec!["safe, fast, productive."],
            search(query, contents, false)
        )
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(vec!["Rust:", "Trust me."], search(query, contents, false));
    }

    #[test]

    fn case_sensitive() {
        let query = "HELLO";
        let contents = "\
HELLO FROM THE OTHER SIDE.
HELLO THERE buDDY
yeah I guess hElLo.
hello hello hello";

        assert_eq!(
            vec!["HELLO FROM THE OTHER SIDE.", "HELLO THERE buDDY"],
            search(query, contents, true)
        );
    }
}
