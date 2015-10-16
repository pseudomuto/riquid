use std::cell::Cell;
use std::cmp;

use regex::{Captures,Regex};

pub struct Scanner<'t> {
    source: &'t str,
    index: Cell<usize>,
    length: usize
}

impl<'t> Scanner<'t> {
    pub fn new<'a>(source: &'a str) -> Scanner<'a> {
        Scanner {
            source: source,
            index: Cell::new(0),
            length: source.len()
        }
    }

    pub fn position(&self) -> usize {
        cmp::min(self.index.get(), self.length)
    }

    pub fn is_eos(&self) -> bool {
        self.position() == self.length
    }

    pub fn skip(&self, n: usize) {
        let pos = cmp::min(self.position() + n, self.length);
        self.index.set(pos);
    }

    pub fn rest(&self) -> Option<&str> {
        if self.is_eos() { return None; }
        Some(self.raw())
    }

    pub fn get_char(&self) -> Option<&str> {
        if self.is_eos() { return None; }

        let rest = self.raw();
        let chr  = &rest[0..1];
        self.skip(chr.len());

        Some(chr)
    }

    pub fn scan(&self, pattern: &Regex) -> Option<&str> {
        self.skip_whitespace();
        let rest = self.raw();

        pattern.captures(rest).and_then(|captures| self.get_match(rest, &captures))
    }

    pub fn check(&self, pattern: &Regex) -> bool {
        self.skip_whitespace();
        let rest = self.raw();

        pattern.captures(rest).is_some()
    }

    fn skip_whitespace(&self) {
        self.skip(self.leading_chars(self.raw()));
    }

    fn get_match<'a>(&'a self, source: &'a str, captures: &Captures) -> Option<&str> {
        captures
            .pos(0)
            .and_then(|(_, count)| {
                let matched   = &source[0..count];
                let remaining = &source[count..];

                self.skip(count + self.leading_chars(remaining));
                Some(matched)
            })
    }

    fn leading_chars(&self, string: &str) -> usize {
        string.len() - string.trim_left_matches(char::is_whitespace).len()
    }

    fn raw(&self) -> &str {
        &self.source[self.position()..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn new_returns_a_new_scanner() {
        let scanner = Scanner::new("test string");
        assert_eq!("test string", scanner.rest().unwrap());
        assert_eq!(0, scanner.position());
    }

    #[test]
    fn rest_returns_all_characters_from_the_current_position() {
        let scanner = Scanner::new("test string");
        assert_eq!("test string", scanner.rest().unwrap());

        scanner.skip(5);
        assert_eq!("string", scanner.rest().unwrap());
    }

    #[test]
    fn is_eos_when_not_at_the_end_of_a_string() {
        let scanner = Scanner::new("test string");
        assert_eq!(false, scanner.is_eos());

        scanner.skip(4);
        assert_eq!(false, scanner.is_eos());
    }

    #[test]
    fn is_eos_is_true_when_at_the_end_of_a_string() {
        let scanner = Scanner::new("test");
        scanner.skip(4);

        assert_eq!(true, scanner.is_eos());
    }

    #[test]
    fn skip_moves_n_characters_ahead() {
        let scanner = Scanner::new("test");
        assert_eq!(0, scanner.position());

        scanner.skip(1);
        assert_eq!(1, scanner.position());

        scanner.skip(2);
        assert_eq!(3, scanner.position());
    }

    #[test]
    fn get_char_returns_the_current_character_and_pushes_the_index() {
        let scanner = Scanner::new("test");
        assert_eq!("t", scanner.get_char().unwrap());
        assert_eq!("e", scanner.get_char().unwrap());
        assert_eq!("s", scanner.get_char().unwrap());
        assert_eq!("t", scanner.get_char().unwrap());
    }

    #[test]
    fn get_char_when_eos_returns_none() {
        let scanner = Scanner::new("test");
        scanner.skip(4);

        assert_eq!(None, scanner.get_char());
    }

    #[test]
    fn scan_retrieves_tokens_from_the_current_position_until_the_end() {
        let pattern = Regex::new(r"^\w+").unwrap();
        let scanner = Scanner::new("test string words");
        assert_eq!("test", scanner.scan(&pattern).unwrap());
        assert_eq!("string", scanner.scan(&pattern).unwrap());
        assert_eq!("words", scanner.scan(&pattern).unwrap());
        assert_eq!(None, scanner.scan(&pattern));
    }

    #[test]
    fn scan_returns_none_when_not_found() {
        let pattern = Regex::new(r"^\d+").unwrap();
        let scanner = Scanner::new("test string");
        assert_eq!(None, scanner.scan(&pattern));
    }

    #[test]
    fn scan_returns_none_when_is_eos() {
        let pattern = Regex::new(r"^\w+").unwrap();
        let scanner = Scanner::new("");
        assert_eq!(None, scanner.scan(&pattern));
    }

    #[test]
    fn scan_returns_none_when_only_whitespace_characters_remain() {
        let pattern = Regex::new(r"^\w+").unwrap();
        let scanner = Scanner::new("  \t \r\n ");
        assert_eq!(None, scanner.scan(&pattern));
        assert!(scanner.is_eos())
    }
}
