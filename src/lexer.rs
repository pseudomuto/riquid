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
    Comparison,
    Identifier,
    Number,
    String,
    Range,
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

pub type LexedToken = (Token, String);

macro_rules! token {
    (Range)                         => (token!(Range, ".."));
    (Pipe)                          => (token!(Pipe, "|"));
    (Dot)                           => (token!(Dot, "."));
    (Colon)                         => (token!(Colon, ":"));
    (Comma)                         => (token!(Comma, ","));
    (OpenSquare)                    => (token!(OpenSquare, "["));
    (CloseSquare)                   => (token!(CloseSquare, "]"));
    (OpenRound)                     => (token!(OpenRound, "("));
    (CloseRound)                    => (token!(CloseRound, ")"));
    (Question)                      => (token!(Question, "?"));
    (Dash)                          => (token!(Dash, "-"));
    ($tokenType:ident, $value:expr) => ((Token::$tokenType, String::from($value)));
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

    fn token_for(&self, pattern: &Regex, value: &str) -> LexedToken {
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

    fn next_match(&self) -> Option<LexedToken> {
        self.matchers.iter().find(|&m| self.scanner.check(m))
            .and_then(|regex| self.matched_token(&regex))
            .or_else(|| self.matched_special())
    }

    fn matched_token(&self, pattern: &Regex) -> Option<LexedToken> {
        let value = self.scanner.scan(pattern).unwrap();
        Some(self.token_for(pattern, value))
    }

    fn matched_special(&self) -> Option<LexedToken> {
        self.scanner.get_char()
            .and_then(|character| {
                self.specials.get(character)
                    .and_then(|token| Some(((*token).clone(), character.into())))
                    .or_else(|| unreachable!("Syntax Error"))
            })
    }
}

impl<'t> Iterator for Tokens<'t> {
    type Item = (Token, String);

    fn next(&mut self) -> Option<LexedToken> {
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

    fn compare_tokens(lexer: &Lexer, expected_tokens: Vec<LexedToken>) {
        let zipped = lexer.tokens().zip(expected_tokens);

        for (actual, expected) in zipped {
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn new_creates_a_new_instance() {
        let lexer = Lexer::new("doSomthing | filter");
        assert_eq!("doSomthing | filter", lexer.scanner.rest().unwrap());
    }

    #[test]
    fn tokens_when_given_a_blank_string() {
        let lexer                   = Lexer::new("");
        let tokens: Vec<LexedToken> = lexer.tokens().collect();

        assert_eq!(0, tokens.len());
    }

    #[test]
    fn tokens_when_given_a_whitespace_only_string() {
        let lexer                   = Lexer::new("  \t \n\r ");
        let tokens: Vec<LexedToken> = lexer.tokens().collect();

        assert_eq!(0, tokens.len());
    }

    #[test]
    fn tokens_parses_identifiers() {
        let lexer    = Lexer::new("high five?");
        let expected = vec![
            token!(Identifier, "high"),
            token!(Identifier, "five?")
        ];

        compare_tokens(&lexer, expected);
    }

    #[test]
    fn tokens_knows_that_identifiers_dont_start_with_numbers() {
        let lexer    = Lexer::new("2foo 5.0bar");
        let expected = vec![
            token!(Number, "2"),
            token!(Identifier, "foo"),
            token!(Number, "5.0"),
            token!(Identifier, "bar")
        ];

        compare_tokens(&lexer, expected);
    }

    #[test]
    fn tokens_parses_string_literals() {
        let lexer    = Lexer::new(r#" 'this is a test""' "wat 'lol'" "#);
        let expected = vec![
            token!(String, r#"'this is a test""'"#),
            token!(String, r#""wat 'lol'""#)
        ];

        compare_tokens(&lexer, expected);
    }

    #[test]
    fn tokens_parses_integers() {
        let lexer    = Lexer::new("hi 50");
        let expected = vec![
            token!(Identifier, "hi"),
            token!(Number, "50")
        ];

        compare_tokens(&lexer, expected);
    }

    #[test]
    fn tokens_parses_floats() {
        let lexer    = Lexer::new("hi 5.0");
        let expected = vec![
            token!(Identifier, "hi"),
            token!(Number, "5.0")
        ];

        compare_tokens(&lexer, expected);
    }

    #[test]
    fn tokens_parses_comparisons() {
        let lexer    = Lexer::new("== <> contains");
        let expected = vec![
            token!(Comparison, "=="),
            token!(Comparison, "<>"),
            token!(Comparison, "contains")
        ];

        compare_tokens(&lexer, expected);
    }

    #[test]
    fn tokens_parses_range_operator() {
        let lexer    = Lexer::new("1..10");
        let expected = vec![
            token!(Number, "1"),
            token!(Range),
            token!(Number, "10")
        ];

        compare_tokens(&lexer, expected);
    }

    #[test]
    fn tokens_parses_special_characters() {
        let lexer    = Lexer::new("[hi], (| .:) - ?cool");
        let expected = vec![
            token!(OpenSquare),
            token!(Identifier, "hi"),
            token!(CloseSquare),
            token!(Comma),
            token!(OpenRound),
            token!(Pipe),
            token!(Dot),
            token!(Colon),
            token!(CloseRound),
            token!(Dash),
            token!(Question),
            token!(Identifier, "cool")
        ];

        compare_tokens(&lexer, expected);
    }

    #[test]
    fn tokens_skips_internal_whitespace() {
        let lexer    = Lexer::new("five|\n\t ==");
        let expected = vec![
            token!(Identifier, "five"),
            token!(Pipe),
            token!(Comparison, "==")
        ];

        compare_tokens(&lexer, expected);
    }

    #[test]
    #[should_panic(expected = "Syntax Error")]
    fn tokens_freaks_out_with_syntax_error() {
        let lexer                   = Lexer::new("%");
        let tokens: Vec<LexedToken> = lexer.tokens().collect();
    }
}
