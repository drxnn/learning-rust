#![warn(missing_debug_implementations)]
#[derive(Debug)]
pub struct StrSplit<'haystack, 'delimiter> {
    remainder: Option<&'haystack str>,
    delimiter: &'delimiter str,

}

impl<'haystack, 'delimiter> StrSplit<'haystack, 'delimiter> {
    pub fn new(haystack: &'haystack str, delimiter: &'delimiter str) -> Self {
        Self { 
            remainder: Some(haystack),
            delimiter, 
        }
    } 
}

impl<'haystack> Iterator for StrSplit<'haystack, '_> {
    type Item = &'haystack str;
    fn next(&mut self) -> Option<Self::Item>{
            let remainder = self.remainder.as_mut()?;
        if let Some(next_delim) = remainder.find(self.delimiter) {
            let until_delimiter = &remainder[..next_delim];
            *remainder = &remainder[next_delim + self.delimiter.len()..];
            Some(until_delimiter)
        } else  {
            self.remainder.take()
        } 
    }
}

pub fn until_char<'s>(s: &'s str, c: char) -> &'s str {
    let delim = format!("{}",c);
    StrSplit::new(s, &delim)
    .next()
    .expect("Strsplit always gives at least one result")
} 


#[test]
fn until_char_test() {
    
    assert_eq!(until_char("hello world", 'o'), "hell");
}
#[test] 
fn it_works(){
let haystack = "a b c d e";
let letters = StrSplit::new(haystack, " ");
assert!(letters.eq(vec!["a", "b", "c", "d", "e"].into_iter()));

 
}