use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    LineHead, // ":"
    LineTail, // "-"
    Split,    // "|"
    Name,
    Number,   // "$12"
    LBracket, // "["
    RBracket, // "]"
    LBrace,   // "{"
    RBrace,   // "}"
    LParen,   // "("
    RParen,   // ")"
    Comma,    // ","
    Equal,    // "="
    Ident,    // "#a"
    At,       // "@"
}

const RESERVE_SYMBOL: [char; 15] = [
    ':', '-', '|', '\'', '[', ']', '{', '}', '(', ')', '$', ',', '=', '#', '@',
];

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
}

pub struct Lexer<'a> {
    src: Peekable<Chars<'a>>,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.chars().peekable(),
            line: 1,
        }
    }

    pub fn new_token(&self, ty: TokenType, value: &str) -> Token {
        Token {
            token_type: ty,
            value: value.to_string(),
            line: self.line,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.consume_whitespace();
        let &c = self.src.peek()?;
        match c {
            ':' => {
                self.src.next();
                Some(self.new_token(TokenType::LineHead, ":"))
            }
            '-' => {
                self.src.next();
                Some(self.new_token(TokenType::LineTail, "-"))
            }
            '|' => {
                self.src.next();
                Some(self.new_token(TokenType::Split, "|"))
            }
            '$' => {
                self.src.next();
                Some(self.collect_number())
            }
            '[' => {
                self.src.next();
                Some(self.new_token(TokenType::LBracket, "["))
            }
            ']' => {
                self.src.next();
                Some(self.new_token(TokenType::RBracket, "]"))
            }
            '{' => {
                self.src.next();
                Some(self.new_token(TokenType::LBrace, "{"))
            }
            '}' => {
                self.src.next();
                Some(self.new_token(TokenType::RBrace, "}"))
            }
            '(' => {
                self.src.next();
                Some(self.new_token(TokenType::LParen, "("))
            }
            ')' => {
                self.src.next();
                Some(self.new_token(TokenType::RParen, ")"))
            }
            '=' => {
                self.src.next();
                Some(self.new_token(TokenType::Equal, "="))
            }
            ',' => {
                self.src.next();
                Some(self.new_token(TokenType::Comma, ","))
            }
            '#' => {
                self.src.next();
                Some(self.collect_ident())
            }
            '@' => {
                self.src.next();
                Some(self.new_token(TokenType::At, "@"))
            }
            '\'' | '\"' => Some(self.collect_quoted_name(c)),
            _ => Some(self.collect_plain_name()),
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(&c) = self.src.peek() {
            if c == '\n' {
                self.line += 1;
                self.src.next();
            } else if c.is_whitespace() {
                self.src.next();
            } else {
                break;
            }
        }
    }

    fn collect_quoted_name(&mut self, quote: char) -> Token {
        self.src.next();
        let mut value = String::new();

        while let Some(&c) = self.src.peek() {
            if c == quote {
                self.src.next();
                break;
            }
            value.push(c);
            self.src.next();
        }

        self.new_token(TokenType::Name, &value)
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

        self.new_token(TokenType::Number, &value)
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

        self.new_token(TokenType::Name, &value)
    }

    fn collect_ident(&mut self) -> Token {
        let mut value = String::new();
        while let Some(&c) = self.src.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '?' {
                value.push(c);
                self.src.next();
            } else {
                break;
            }
        }

        self.new_token(TokenType::Ident, &value)
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
mod tests {
    use super::*;

    #[test]
    fn simple_input() {
        let input = ":| A | '|' | 'P' | Back |-";

        let mut lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.tokenization();

        let right_result = vec![
            Token {
                token_type: TokenType::LineHead,
                value: String::from(":"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("A"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("P"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("Back"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::LineTail,
                value: String::from("-"),
                line: 1,
            },
        ];

        assert_eq!(tokens, right_result);
    }

    #[test]
    fn specify_length() {
        let input = ":| A | Back [$10] |-";

        let mut lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.tokenization();

        let right_result = vec![
            Token {
                token_type: TokenType::LineHead,
                value: String::from(":"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("A"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("Back"),
                line: 1,
            },
            Token {
                token_type: TokenType::LBracket,
                value: String::from("["),
                line: 1,
            },
            Token {
                token_type: TokenType::Number,
                value: String::from("10"),
                line: 1,
            },
            Token {
                token_type: TokenType::RBracket,
                value: String::from("]"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::LineTail,
                value: String::from("-"),
                line: 1,
            },
        ];

        assert_eq!(tokens, right_result);
    }

    #[test]
    fn multi_binds() {
        let input = ":| A | B, C, D |-";

        let mut lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.tokenization();

        let right_result = vec![
            Token {
                token_type: TokenType::LineHead,
                value: String::from(":"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("A"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("B"),
                line: 1,
            },
            Token {
                token_type: TokenType::Comma,
                value: String::from(","),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("C"),
                line: 1,
            },
            Token {
                token_type: TokenType::Comma,
                value: String::from(","),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("D"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::LineTail,
                value: String::from("-"),
                line: 1,
            },
        ];

        assert_eq!(tokens, right_result);
    }

    #[test]
    fn assign() {
        let input = "#id = $10 \n#color = @($0, $0, $0)";

        let mut lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.tokenization();

        let right_result = vec![
            Token {
                token_type: TokenType::Ident,
                value: String::from("id"),
                line: 1,
            },
            Token {
                token_type: TokenType::Equal,
                value: String::from("="),
                line: 1,
            },
            Token {
                token_type: TokenType::Number,
                value: String::from("10"),
                line: 1,
            },
            Token {
                token_type: TokenType::Ident,
                value: String::from("color"),
                line: 2,
            },
            Token {
                token_type: TokenType::Equal,
                value: String::from("="),
                line: 2,
            },
            Token {
                token_type: TokenType::At,
                value: String::from("@"),
                line: 2,
            },
            Token {
                token_type: TokenType::LParen,
                value: String::from("("),
                line: 2,
            },
            Token {
                token_type: TokenType::Number,
                value: String::from("0"),
                line: 2,
            },
            Token {
                token_type: TokenType::Comma,
                value: String::from(","),
                line: 2,
            },
            Token {
                token_type: TokenType::Number,
                value: String::from("0"),
                line: 2,
            },
            Token {
                token_type: TokenType::Comma,
                value: String::from(","),
                line: 2,
            },
            Token {
                token_type: TokenType::Number,
                value: String::from("0"),
                line: 2,
            },
            Token {
                token_type: TokenType::RParen,
                value: String::from(")"),
                line: 2,
            },
        ];

        assert_eq!(tokens, right_result);
    }

    #[test]
    fn customized_name() {
        let input = ":| '^' {'up'} |-";

        let mut lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.tokenization();

        let right_result = vec![
            Token {
                token_type: TokenType::LineHead,
                value: String::from(":"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("^"),
                line: 1,
            },
            Token {
                token_type: TokenType::LBrace,
                value: String::from("{"),
                line: 1,
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("up"),
                line: 1,
            },
            Token {
                token_type: TokenType::RBrace,
                value: String::from("}"),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
                line: 1,
            },
            Token {
                token_type: TokenType::LineTail,
                value: String::from("-"),
                line: 1,
            },
        ];

        assert_eq!(tokens, right_result);
    }
}
