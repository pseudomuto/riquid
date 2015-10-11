use std::collections::HashMap;

use scanner::Scanner;
use regex::Regex;

const COMPARISON           : &'static str = r"^(==|!=|<>|<=?|>=?|contains)";
const SINGLE_STRING_LITERAL: &'static str = r"^'[^']*'";
const DOUBLE_STRING_LITERAL: &'static str = r#"^"[^"]*""#;
const NUMBER_LITERAL:        &'static str = r"^-?\d+(\.\d+)?";
const IDENTIFIER:            &'static str = r"^[a-zA-Z_][\w-]*\??";
const RANGE_OP:              &'static str = r"^\.\.";

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Comparison(String),
    Identifier(String),
    Number(String),
    String(String),
    Range(String),
    Pipe,
    Dot,
    Colon,
    Comma,
    OpenSquare,
    CloseSquare,
    OpenRound,
    CloseRound,
    Question,
    Dash
}

macro_rules! token {
    (Range)               => (token!(Range, ".."));
    ($e:ident)            => (Token::$e);
    ($e:ident, $str:expr) => (Token::$e(String::from($str)))
}

pub struct Tokens<'t> {
    scanner: &'t Scanner<'t>,
    specials: HashMap<&'t str, Token>,
    matchers: Vec<Regex>
}

impl<'t> Tokens<'t> {
    fn new<'a>(scanner: &'a Scanner<'a>) -> Tokens<'a> {
        let mut specials = HashMap::new();
        specials.insert("|", Token::Pipe);
        specials.insert(".", Token::Dot);
        specials.insert(":", Token::Colon);
        specials.insert(",", Token::Comma);
        specials.insert("[", Token::OpenSquare);
        specials.insert("]", Token::CloseSquare);
        specials.insert("(", Token::OpenRound);
        specials.insert(")", Token::CloseRound);
        specials.insert("?", Token::Question);
        specials.insert("-", Token::Dash);

        let matchers = vec![
            Regex::new(COMPARISON).unwrap(),
            Regex::new(SINGLE_STRING_LITERAL).unwrap(),
            Regex::new(DOUBLE_STRING_LITERAL).unwrap(),
            Regex::new(NUMBER_LITERAL).unwrap(),
            Regex::new(IDENTIFIER).unwrap(),
            Regex::new(RANGE_OP).unwrap()
        ];

        Tokens { scanner: scanner, specials: specials, matchers: matchers }
    }

    fn token_for(&self, pattern: &Regex, value: &str) -> Token {
        match pattern.as_str() {
            COMPARISON            => token!(Comparison, value),
            SINGLE_STRING_LITERAL => token!(String, value),
            DOUBLE_STRING_LITERAL => token!(String, value),
            NUMBER_LITERAL        => token!(Number, value),
            IDENTIFIER            => token!(Identifier, value),
            RANGE_OP              => token!(Range),
            _                     => unreachable!() // already been checked for existence
        }
    }

    fn next_match(&self) -> Option<Token> {
        match self.matchers.iter().find(|&m| self.scanner.check(m)) {
            Some(regex) => self.matched_token(&regex),
            None => self.matched_special()
        }
    }

    fn matched_token(&self, pattern: &Regex) -> Option<Token> {
        let value = self.scanner.scan(pattern).unwrap();
        Some(self.token_for(pattern, value))
    }

    fn matched_special(&self) -> Option<Token> {
        match self.scanner.get_char() {
            Some(character) => {
                match self.specials.get(character) {
                    Some(token) => Some((*token).clone()),
                    None => unreachable!("Syntax Error")
                }
            },
            None => None
        }
    }
}

impl<'t> Iterator for Tokens<'t> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.next_match()
    }
}

pub struct Lexer<'t> {
    scanner: Scanner<'t>
}

impl<'t> Lexer<'t> {
    pub fn new<'a>(source: &'a str) -> Lexer<'a> {
        Lexer { scanner: Scanner::new(source) }
    }

    pub fn tokens(&self) -> Tokens {
        Tokens::new(&self.scanner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_a_new_instance() {
        let lexer = Lexer::new("doSomthing | filter");
        assert_eq!("doSomthing | filter", lexer.scanner.rest().unwrap());
    }

    #[test]
    fn tokens_when_given_a_blank_string() {
        let lexer              = Lexer::new("");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(0, tokens.len());
    }

    #[test]
    fn tokens_when_given_a_whitespace_only_string() {
        let lexer              = Lexer::new("  \t \n\r ");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(0, tokens.len());
    }

    #[test]
    fn tokens_parses_identifiers() {
        let lexer              = Lexer::new("high five?");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(2, tokens.len());
        assert_eq!(token!(Identifier, "high"), tokens[0]);
        assert_eq!(token!(Identifier, "five?"), tokens[1]);
    }

    #[test]
    fn tokens_knows_that_identifiers_dont_start_with_numbers() {
        let lexer              = Lexer::new("2foo 5.0bar");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(4, tokens.len());
        assert_eq!(token!(Number, "2"), tokens[0]);
        assert_eq!(token!(Identifier, "foo"), tokens[1]);
        assert_eq!(token!(Number, "5.0"), tokens[2]);
        assert_eq!(token!(Identifier, "bar"), tokens[3]);
    }

    #[test]
    fn tokens_parses_string_literals() {
        let lexer              = Lexer::new(r#" 'this is a test""' "wat 'lol'" "#);
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(2, tokens.len());
        assert_eq!(token!(String, r#"'this is a test""'"#), tokens[0]);
        assert_eq!(token!(String, r#""wat 'lol'""#), tokens[1]);
    }

    #[test]
    fn tokens_parses_integers() {
        let lexer              = Lexer::new("hi 50");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(2, tokens.len());
        assert_eq!(token!(Identifier, "hi"), tokens[0]);
        assert_eq!(token!(Number, "50"), tokens[1]);
    }

    #[test]
    fn tokens_parses_floats() {
        let lexer              = Lexer::new("hi 5.0");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(2, tokens.len());
        assert_eq!(token!(Identifier, "hi"), tokens[0]);
        assert_eq!(token!(Number, "5.0"), tokens[1]);
    }

    #[test]
    fn tokens_parses_comparisons() {
        let lexer              = Lexer::new("== <> contains");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(3, tokens.len());
        assert_eq!(token!(Comparison, "=="), tokens[0]);
        assert_eq!(token!(Comparison, "<>"), tokens[1]);
        assert_eq!(token!(Comparison, "contains"), tokens[2]);
    }

    #[test]
    fn tokens_parses_range_operator() {
        let lexer              = Lexer::new("1..10");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(3, tokens.len());
        assert_eq!(token!(Number, "1"), tokens[0]);
        assert_eq!(token!(Range), tokens[1]);
        assert_eq!(token!(Number, "10"), tokens[2]);
    }

    #[test]
    fn tokens_parses_special_characters() {
        let lexer              = Lexer::new("[hi], (| .:) - ?cool");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(12, tokens.len());
        assert_eq!(token!(OpenSquare), tokens[0]);
        assert_eq!(token!(Identifier, "hi"), tokens[1]);
        assert_eq!(token!(CloseSquare), tokens[2]);
        assert_eq!(token!(Comma), tokens[3]);
        assert_eq!(token!(OpenRound), tokens[4]);
        assert_eq!(token!(Pipe), tokens[5]);
        assert_eq!(token!(Dot), tokens[6]);
        assert_eq!(token!(Colon), tokens[7]);
        assert_eq!(token!(CloseRound), tokens[8]);
        assert_eq!(token!(Dash), tokens[9]);
        assert_eq!(token!(Question), tokens[10]);
        assert_eq!(token!(Identifier, "cool"), tokens[11]);
    }

    #[test]
    fn tokens_skips_internal_whitespace() {
        let lexer              = Lexer::new("five|\n\t  ==");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(3, tokens.len());
        assert_eq!(token!(Identifier, "five"), tokens[0]);
        assert_eq!(token!(Pipe), tokens[1]);
        assert_eq!(token!(Comparison, "=="), tokens[2]);
    }

    #[test]
    #[should_panic(expected = "Syntax Error")]
    fn tokens_freaks_out_with_syntax_error() {
        let lexer              = Lexer::new("%");
        let tokens: Vec<Token> = lexer.tokens().collect();
    }
}
