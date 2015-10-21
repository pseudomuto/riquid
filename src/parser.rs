use lexer::{LexedToken,Lexer,Token};

pub struct Parser {
    tokens: Vec<LexedToken>,
    current_index: usize
}

impl Parser {
    pub fn new(source: &str) -> Parser {
        let lexer = Lexer::new(source);
        Parser { tokens: lexer.tokens().collect(), current_index: 0 }
    }

    pub fn jump(&mut self, n: isize) {
        match self.index_with_offset(n) {
            Some(index) => self.current_index = index,
            None => panic!("Attempted to jump too far back")
        }
    }

    pub fn consume(&mut self, token: Token) -> Option<String> {
        self.token_at(self.current_index)
            .and_then(|&(ref token_type, ref value)| {
                if *token_type != token { return None; }
                Some(value.clone().into())
            })
            .and_then(|string| {
                self.current_index += 1;
                Some(string)
            })
    }

    pub fn expression(&mut self) -> Option<String> {
        self.type_at(self.current_index)
            .and_then(|token_type| {
                match token_type {
                    Token::Identifier => self.variable(),
                    Token::OpenRound => self.range(),
                    Token::String | Token::Number => self.consume(token_type),
                    _ => panic!("Syntax Error")
                }
            })
    }

    pub fn is_current(&self, token: Token) -> bool {
        self.is_current_offset(token, 0)
    }

    pub fn is_current_offset(&self, token: Token, offset: isize) -> bool {
        self.index_with_offset(offset)
            .and_then(|index| self.is_token(index, token))
            .or_else(|| Some(false))
            .unwrap()
    }

    fn token_at(&self, index: usize) -> Option<&LexedToken> {
        self.tokens.get(index)
    }

    fn type_at(&self, index: usize) -> Option<Token> {
        self.token_at(index)
            .and_then(|&(ref token, _)| Some(token.clone()))
    }

    fn is_token(&self, index: usize, token: Token) -> Option<bool> {
        self.token_at(index)
            .and_then(|&(ref token_type, _)| Some(*token_type == token))
            .or_else(|| Some(false))
    }

    fn index_with_offset(&self, offset: isize) -> Option<usize> {
        let index = (self.current_index as isize).wrapping_add(offset);
        if index < 0 { return None; }

        Some(index as usize)
    }

    fn variable(&mut self) -> Option<String> {
        self.consume(Token::Identifier)
            .and_then(|mut value| {
                while self.is_current(Token::OpenSquare) {
                    value.push_str(&self.consume(Token::OpenSquare).unwrap());
                    value.push_str(&self.expression().unwrap());
                    value.push_str(&self.consume(Token::CloseSquare).unwrap());
                }

                if self.is_current(Token::Dot) {
                    value.push_str(&self.consume(Token::Dot).unwrap());
                    value.push_str(&self.variable().unwrap());
                }

                Some(value)
            })
    }

    fn range(&mut self) -> Option<String> {
        self.consume(Token::OpenRound)
            .and_then(|mut value| {
                value.push_str(&self.expression().unwrap());
                value.push_str(&self.consume(Token::Range).unwrap());
                value.push_str(&self.expression().unwrap());
                value.push_str(&self.consume(Token::CloseRound).unwrap());
                Some(value)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::Token;

    #[test]
    fn jump_moves_the_current_index() {
        let mut parser = Parser::new("wat: 7");
        parser.jump(2);

        assert!(parser.is_current(Token::Number));
    }

    #[test]
    fn jump_can_move_backwards() {
        let mut parser = Parser::new("wat: 7");
        parser.jump(2);
        parser.jump(-1);

        assert!(parser.is_current(Token::Colon));
    }

    #[test]
    #[should_panic(expected="Attempted to jump too far back")]
    fn jump_panics_when_index_goes_below_zero() {
        let mut parser = Parser::new("wat: 7");
        parser.jump(-1)
    }

    #[test]
    fn consume_things() {
        let mut parser = Parser::new("wat: 7");
        assert_eq!("wat", parser.consume(Token::Identifier).unwrap());
        assert_eq!(":", parser.consume(Token::Colon).unwrap());
        assert_eq!("7", parser.consume(Token::Number).unwrap());
    }

    #[test]
    fn consume_returns_none_when_token_doesnt_match() {
        let mut parser = Parser::new("wat: 7");
        assert_eq!(None, parser.consume(Token::Number));
        assert_eq!(None, parser.consume(Token::Colon));
        assert!(parser.consume(Token::Identifier).is_some());
    }

    #[test]
    fn is_current_checks_token_type() {
        let mut parser = Parser::new("wat 6 Peter Hegemon");

        assert!(parser.is_current(Token::Identifier));
        parser.consume(Token::Identifier);

        assert_eq!(false, parser.is_current(Token::Comparison));
        assert!(parser.is_current(Token::Number));
        assert!(parser.is_current_offset(Token::Identifier, 1));
        assert_eq!(false, parser.is_current_offset(Token::Number, 1));
    }

    #[test]
    fn is_current_offset_returns_false_when_offset_is_not_valid() {
        let mut parser = Parser::new("wat 6 Peter Hegemon");
        parser.jump(1);

        assert!(parser.is_current_offset(Token::Number, 0));
        assert!(parser.is_current_offset(Token::Identifier, -1));
        assert_eq!(false, parser.is_current_offset(Token::Identifier, -2));
    }

    #[test]
    fn expression_parsing_identifiers_strings_and_numbers() {
        let mut parser = Parser::new("hi.there hi?[5].there? hi.there.bob");
        assert_eq!("hi.there", parser.expression().unwrap());
        assert_eq!("hi?[5].there?", parser.expression().unwrap());
        assert_eq!("hi.there.bob", parser.expression().unwrap());

        let mut parser = Parser::new("567 6.0 'lol' \"wut\"");
        assert_eq!("567", parser.expression().unwrap());
        assert_eq!("6.0", parser.expression().unwrap());
        assert_eq!("'lol'", parser.expression().unwrap());
        assert_eq!("\"wut\"", parser.expression().unwrap());
    }

    #[test]
    fn expression_parsing_ranges() {
        let mut parser = Parser::new("(5..7) (1.5..9.6) (young..old) (hi[5].wat..old)");
        assert_eq!("(5..7)", parser.expression().unwrap());
        assert_eq!("(1.5..9.6)", parser.expression().unwrap());
        assert_eq!("(young..old)", parser.expression().unwrap());
        assert_eq!("(hi[5].wat..old)", parser.expression().unwrap());
    }
}
