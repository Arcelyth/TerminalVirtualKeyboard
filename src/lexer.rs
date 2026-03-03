use std::iter::Peekable;
use std::str::Chars;


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    LineHead,   // ":"
    LineTail,   // "-"
    Split,      // "|"
    Name,     
    Number,     // "$12"
    LBracket,   // "["
    RBracket,   // "]"
    LBrace,     // "{"
    RBrace,     // "}"
}

const RESERVE_SYMBOL: [char; 9] = [':', '-', '|', '\'', '[', ']', '{', '}', '$'];

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

pub struct Lexer<'a> {
    src: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src: src.chars().peekable() }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.consume_whitespace();
        let &c = self.src.peek()?;
        match c {
            ':' => {
                self.src.next();
                Some(Token { token_type: TokenType::LineHead, value: ":".to_string() })
            }
            '-' => {
                self.src.next();
                Some(Token { token_type: TokenType::LineTail, value: "-".to_string() })
            }
            '|' => {
                self.src.next();
                Some(Token { token_type: TokenType::Split, value: "|".to_string() })
            }
            '$' => {
                self.src.next();
                Some(self.collect_number())
            }
            '[' => {
                self.src.next();
                Some(Token { token_type: TokenType::LBracket, value: "[".to_string() })
            }
            ']' => {
                self.src.next();
                Some(Token { token_type: TokenType::RBracket, value: "]".to_string() })
            }
            '{' => {
                self.src.next();
                Some(Token { token_type: TokenType::LBrace, value: "{".to_string() })
            }
            '}' => {
                self.src.next();
                Some(Token { token_type: TokenType::RBrace, value: "}".to_string() })
            }
            '\'' => {
                Some(self.collect_quoted_name())
            }
            _ => {
                Some(self.collect_plain_name())
            }
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(&c) = self.src.peek() {
            if c.is_whitespace() {
                self.src.next();
            } else {
                break;
            }
        }
    }

    fn collect_quoted_name(&mut self) -> Token {
        self.src.next(); 
        let mut value = String::new();
        
        while let Some(&c) = self.src.peek() {
            if c == '\'' {
                self.src.next(); 
                break;
            }
            value.push(c);
            self.src.next();
        }
        
        Token { token_type: TokenType::Name, value }
    }

    fn collect_number(&mut self) -> Token {
        let mut value = String::new();
        while let Some(&c) = self.src.peek() {
            if c.is_numeric() && !RESERVE_SYMBOL.contains(&c) {
                value.push(c);
                self.src.next();
            } else {
                break;
            }
        }
        Token { token_type: TokenType::Number, value }
    }

    fn collect_plain_name(&mut self) -> Token {
        let mut value = String::new();
        while let Some(&c) = self.src.peek() {
            if c.is_whitespace() || RESERVE_SYMBOL.contains(&c) {
                break;
            }
            value.push(c);
            self.src.next();
        }
        Token { token_type: TokenType::Name, value }
    }

    pub fn tokenization(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn simple_input(){
        let input = ":| A | '|' | 'P' | Back |-";
    
        let mut lexer = Lexer::new(input);
        let tokens:Vec<Token> = lexer.tokenization();

        let right_result = vec![
            Token { token_type: TokenType::LineHead, value: String::from(":") },
            Token { token_type: TokenType::Split, value: String::from("|") },
            Token { token_type: TokenType::Name, value: String::from("A")},
            Token { token_type: TokenType::Split, value: String::from("|") },
            Token { token_type: TokenType::Name, value: String::from("|") },
            Token { token_type: TokenType::Split, value: String::from("|")},
            Token { token_type: TokenType::Name, value: String::from("P") },
            Token { token_type: TokenType::Split, value: String::from("|") },
            Token { token_type: TokenType::Name, value: String::from("Back") },
            Token { token_type: TokenType::Split, value: String::from("|") },
            Token { token_type: TokenType::LineTail, value: String::from("-") },

        ];     

        assert_eq!(tokens, right_result);
    }

    #[test]
    fn specify_length(){
        let input = ":| A | Back [$10] |-";
    
        let mut lexer = Lexer::new(input);
        let tokens:Vec<Token> = lexer.tokenization();

        let right_result = vec![
            Token { token_type: TokenType::LineHead, value: String::from(":") },
            Token { token_type: TokenType::Split, value: String::from("|") },
            Token { token_type: TokenType::Name, value: String::from("A")},
            Token { token_type: TokenType::Split, value: String::from("|") },
            Token { token_type: TokenType::Name, value: String::from("Back") },
            Token { token_type: TokenType::LBracket, value: String::from("[") },
            Token { token_type: TokenType::Number, value: String::from("10") },
            Token { token_type: TokenType::RBracket, value: String::from("]") },
            Token { token_type: TokenType::Split, value: String::from("|") },
            Token { token_type: TokenType::LineTail, value: String::from("-") },
        ];     

        assert_eq!(tokens, right_result);
    }

   
}
