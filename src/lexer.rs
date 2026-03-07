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
}

pub struct Lexer<'a> {
    src: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.consume_whitespace();
        let &c = self.src.peek()?;
        match c {
            ':' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::LineHead,
                    value: ":".to_string(),
                })
            }
            '-' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::LineTail,
                    value: "-".to_string(),
                })
            }
            '|' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::Split,
                    value: "|".to_string(),
                })
            }
            '$' => {
                self.src.next();
                Some(self.collect_number())
            }
            '[' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::LBracket,
                    value: "[".to_string(),
                })
            }
            ']' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::RBracket,
                    value: "]".to_string(),
                })
            }
            '{' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::LBrace,
                    value: "{".to_string(),
                })
            }
            '}' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::RBrace,
                    value: "}".to_string(),
                })
            }
            '(' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::LParen,
                    value: "(".to_string(),
                })
            }
            ')' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::RParen,
                    value: ")".to_string(),
                })
            }
            '=' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::Equal,
                    value: "=".to_string(),
                })
            }
            ',' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::Comma,
                    value: ",".to_string(),
                })
            }
            '#' => {
                self.src.next();
                Some(self.collect_ident())
            }
            '@' => {
                self.src.next();
                Some(Token {
                    token_type: TokenType::At,
                    value: "@".to_string(),
                })
            }
            '\'' | '\"' => Some(self.collect_quoted_name(c)),
            _ => Some(self.collect_plain_name()),
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

        Token {
            token_type: TokenType::Name,
            value,
        }
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
        Token {
            token_type: TokenType::Number,
            value,
        }
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
        Token {
            token_type: TokenType::Name,
            value,
        }
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
        Token {
            token_type: TokenType::Ident,
            value,
        }
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
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("A"),
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("P"),
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("Back"),
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::LineTail,
                value: String::from("-"),
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
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("A"),
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("Back"),
            },
            Token {
                token_type: TokenType::LBracket,
                value: String::from("["),
            },
            Token {
                token_type: TokenType::Number,
                value: String::from("10"),
            },
            Token {
                token_type: TokenType::RBracket,
                value: String::from("]"),
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::LineTail,
                value: String::from("-"),
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
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("A"),
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("B"),
            },
            Token {
                token_type: TokenType::Comma,
                value: String::from(","),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("C"),
            },
            Token {
                token_type: TokenType::Comma,
                value: String::from(","),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("D"),
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::LineTail,
                value: String::from("-"),
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
            },
            Token {
                token_type: TokenType::Equal,
                value: String::from("="),
            },
            Token {
                token_type: TokenType::Number,
                value: String::from("10"),
            },
            Token {
                token_type: TokenType::Ident,
                value: String::from("color"),
            },
            Token {
                token_type: TokenType::Equal,
                value: String::from("="),
            },
            Token {
                token_type: TokenType::At,
                value: String::from("@"),
            },
            Token {
                token_type: TokenType::LParen,
                value: String::from("("),
            },
            Token {
                token_type: TokenType::Number,
                value: String::from("0"),
            },
            Token {
                token_type: TokenType::Comma,
                value: String::from(","),
            },
            Token {
                token_type: TokenType::Number,
                value: String::from("0"),
            },
            Token {
                token_type: TokenType::Comma,
                value: String::from(","),
            },
            Token {
                token_type: TokenType::Number,
                value: String::from("0"),
            },
            Token {
                token_type: TokenType::RParen,
                value: String::from(")"),
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
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("^"),
            },
            Token {
                token_type: TokenType::LBrace,
                value: String::from("{"),
            },
            Token {
                token_type: TokenType::Name,
                value: String::from("up"),
            },
            Token {
                token_type: TokenType::RBrace,
                value: String::from("}"),
            },
            Token {
                token_type: TokenType::Split,
                value: String::from("|"),
            },
            Token {
                token_type: TokenType::LineTail,
                value: String::from("-"),
            },
        ];

        assert_eq!(tokens, right_result);
    }
}
