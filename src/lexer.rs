use scanner::Scanner;
use regex::Regex;

const COMPARISON           : &'static str = r"^(==|!=|<>|<=?|>=?|contains)";
const SINGLE_STRING_LITERAL: &'static str = r"^'[^']*'";
const DOUBLE_STRING_LITERAL: &'static str = r#"^"[^"]*""#;
const NUMBER_LITERAL:        &'static str = r"^-?\d+(\.\d+)?";
const IDENTIFIER:            &'static str = r"^[a-zA-Z_][\w-]*\??";
const RANGE_OP:              &'static str = r"^\.\.";

#[derive(Debug, PartialEq)]
pub enum Token {
    Comparison(String),
    EndOfString,
    Identifier(String),
    Number(String),
    Range,
    String(String),
}

pub struct Tokens<'t> {
    scanner: &'t Scanner<'t>
}

impl<'t> Tokens<'t> {
    fn token_for(&self, pattern: &Regex, value: &str) -> Token {
        match pattern.as_str() {
            COMPARISON => Token::Comparison(value.to_string()),
            SINGLE_STRING_LITERAL => Token::String(value.to_string()),
            DOUBLE_STRING_LITERAL => Token::String(value.to_string()),
            NUMBER_LITERAL => Token::Number(value.to_string()),
            IDENTIFIER => Token::Identifier(value.to_string()),
            RANGE_OP => Token::Range,
            _ => Token::EndOfString
        }
    }
}

impl<'t> Iterator for Tokens<'t> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let matchers = vec![
            Regex::new(COMPARISON).unwrap(),
            Regex::new(SINGLE_STRING_LITERAL).unwrap(),
            Regex::new(DOUBLE_STRING_LITERAL).unwrap(),
            Regex::new(NUMBER_LITERAL).unwrap(),
            Regex::new(IDENTIFIER).unwrap(),
            Regex::new(RANGE_OP).unwrap()
        ];

        match matchers.iter().find(|&m| self.scanner.check(m)) {
            None => None,
            Some(regex) => {
                let value = self.scanner.scan(&regex).unwrap();
                Some(self.token_for(&regex, &value))
            }
        }
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
        Tokens { scanner: &self.scanner }
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
        assert_eq!(Token::Identifier("high".to_string()), tokens[0]);
        assert_eq!(Token::Identifier("five?".to_string()), tokens[1]);
    }

    #[test]
    fn tokens_knows_that_identifiers_dont_start_with_numbers() {
        let lexer              = Lexer::new("2foo 5.0bar");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(4, tokens.len());
        assert_eq!(Token::Number("2".to_string()), tokens[0]);
        assert_eq!(Token::Identifier("foo".to_string()), tokens[1]);
        assert_eq!(Token::Number("5.0".to_string()), tokens[2]);
        assert_eq!(Token::Identifier("bar".to_string()), tokens[3]);
    }

    #[test]
    fn tokens_parses_string_literals() {
        let lexer              = Lexer::new(r#" 'this is a test""' "wat 'lol'" "#);
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(2, tokens.len());
        assert_eq!(Token::String(r#"'this is a test""'"#.to_string()), tokens[0]);
        assert_eq!(Token::String(r#""wat 'lol'""#.to_string()), tokens[1]);
    }

    #[test]
    fn tokens_parses_integers() {
        let lexer              = Lexer::new("hi 50");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(2, tokens.len());
        assert_eq!(Token::Identifier("hi".to_string()), tokens[0]);
        assert_eq!(Token::Number("50".to_string()), tokens[1]);
    }

    #[test]
    fn tokens_parses_floats() {
        let lexer              = Lexer::new("hi 5.0");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(2, tokens.len());
        assert_eq!(Token::Identifier("hi".to_string()), tokens[0]);
        assert_eq!(Token::Number("5.0".to_string()), tokens[1]);
    }

    #[test]
    fn tokens_parses_comparisons() {
        let lexer              = Lexer::new("== <> contains");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(3, tokens.len());
        assert_eq!(Token::Comparison("==".to_string()), tokens[0]);
        assert_eq!(Token::Comparison("<>".to_string()), tokens[1]);
        assert_eq!(Token::Comparison("contains".to_string()), tokens[2]);
    }

    #[test]
    fn tokens_parses_range_operator() {
        let lexer              = Lexer::new("1..10");
        let tokens: Vec<Token> = lexer.tokens().collect();

        assert_eq!(3, tokens.len());
        assert_eq!(Token::Number("1".to_string()), tokens[0]);
        assert_eq!(Token::Range, tokens[1]);
        assert_eq!(Token::Number("10".to_string()), tokens[2]);
    }
}
