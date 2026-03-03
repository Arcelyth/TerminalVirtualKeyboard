use crate::error::*;
use crate::layout::*;
use crate::lexer::*;
use rdev::{EventType, Key, listen};
use std::sync::Arc;
#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

pub struct Attr {
    width: u16,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn peek(&self) -> Result<&Token, ParserError> {
        if self.current < self.tokens.len() {
            Ok(&self.tokens[self.current])
        } else {
            Err(ParserError::Err("EOF".to_string()))
        }
    }

    fn advance(&mut self) -> Result<&Token, ParserError> {
        if self.current < self.tokens.len() {
            self.current += 1;
            Ok(&self.tokens[self.current - 1])
        } else {
            Err(ParserError::Err("EOF".to_string()))
        }
    }

    fn consume(&mut self, ty: TokenType) -> Result<&Token, ParserError> {
        let t = self.peek()?;
        if t.token_type == ty {
            self.advance()
        } else {
            Err(ParserError::Err(format!(
                "Expected {:?}, found {:?}",
                ty, t.token_type
            )))
        }
    }

    pub fn parse(&mut self) -> Result<Layout, ParserError> {
        let mut layer = Vec::new();

        while self.current < self.tokens.len() {
            layer.push(self.parse_line()?);
        }

        Ok(Layout { layer })
    }

    fn parse_line(&mut self) -> Result<Vec<Button>, ParserError> {
        let mut row = Vec::new();
        self.consume(TokenType::LineHead)?;
        self.consume(TokenType::Split)?;

        while self.current < self.tokens.len() && self.peek()?.token_type != TokenType::LineTail {
            let name_token = self.consume(TokenType::Name)?;
            let name_str = name_token.value.clone();

            let mut attr = get_default_width(&name_str);

            if self.peek()?.token_type == TokenType::LBracket {
                self.parse_attr(&mut attr)?;
            }

            row.push(Button {
                rdev_key: get_rdev_key(&name_str),
                width: attr.width,
                name: Arc::from(name_str.as_str()),
            });

            self.consume(TokenType::Split)?;
        }

        self.consume(TokenType::LineTail)?;
        Ok(row)
    }

    fn parse_attr(&mut self, attr: &mut Attr) -> Result<(), ParserError> {
        self.consume(TokenType::LBracket)?;

        while self.peek()?.token_type != TokenType::RBracket {
            match self.peek()?.token_type {
                TokenType::Number => self.parse_width(attr)?,
                _ => return Err(ParserError::Err("Invalid attribute".to_string())),
            }
        }

        self.consume(TokenType::RBracket)?;
        Ok(())
    }

    fn parse_width(&mut self, attr: &mut Attr) -> Result<(), ParserError> {
        let num_token = self.consume(TokenType::Number)?;
        attr.width = num_token
            .value
            .parse::<u16>()
            .map_err(|_| ParserError::Err("Invalid number format".to_string()))?;
        Ok(())
    }
}

fn get_rdev_key(name: &str) -> Option<Key> {
    match name.to_lowercase().as_str() {
        "esc" | "escape" => Some(Key::Escape),
        "1" => Some(Key::Num1),
        "2" => Some(Key::Num2),
        "3" => Some(Key::Num3),
        "4" => Some(Key::Num4),
        "5" => Some(Key::Num5),
        "6" => Some(Key::Num6),
        "7" => Some(Key::Num7),
        "8" => Some(Key::Num8),
        "9" => Some(Key::Num9),
        "0" => Some(Key::Num0),
        "back" | "backspace" => Some(Key::Backspace),
        "tab" => Some(Key::Tab),
        "q" => Some(Key::KeyQ),
        "w" => Some(Key::KeyW),
        "e" => Some(Key::KeyE),
        "r" => Some(Key::KeyR),
        "t" => Some(Key::KeyT),
        "y" => Some(Key::KeyY),
        "u" => Some(Key::KeyU),
        "i" => Some(Key::KeyI),
        "o" => Some(Key::KeyO),
        "p" => Some(Key::KeyP),
        "enter" | "return" => Some(Key::Return),
        "caps" | "capslock" => Some(Key::CapsLock),
        "a" => Some(Key::KeyA),
        "s" => Some(Key::KeyS),
        "d" => Some(Key::KeyD),
        "f" => Some(Key::KeyF),
        "g" => Some(Key::KeyG),
        "h" => Some(Key::KeyH),
        "j" => Some(Key::KeyJ),
        "k" => Some(Key::KeyK),
        "l" => Some(Key::KeyL),
        "lshift" | "shift" => Some(Key::ShiftLeft),
        "rshift" => Some(Key::ShiftRight),
        "z" => Some(Key::KeyZ),
        "x" => Some(Key::KeyX),
        "c" => Some(Key::KeyC),
        "v" => Some(Key::KeyV),
        "b" => Some(Key::KeyN),
        "n" => Some(Key::KeyN),
        "m" => Some(Key::KeyM),
        "ctrl" | "lctrl" => Some(Key::ControlLeft),
        "rctrl" => Some(Key::ControlRight),
        "alt" | "lalt" => Some(Key::Alt),
        "ralt" | "altgr" => Some(Key::AltGr),
        "space" => Some(Key::Space),
        _ => None,
    }
}

fn get_default_width(name: &str) -> Attr {
    match name.to_lowercase().as_str() {
        "space" => Attr { width: 20 },
        _ => Attr { width: 4 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdev::Key;

    // Helper to create a Name token
    fn t_name(val: &str) -> Token {
        Token {
            token_type: TokenType::Name,
            value: val.to_string(),
        }
    }

    #[test]
    fn test_parser_success() {
        // Input sequence for: :| Tab | 'P' | -
        let tokens = vec![
            Token {
                token_type: TokenType::LineHead,
                value: ":".into(),
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
            },
            t_name("Tab"),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
            },
            t_name("P"),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
            },
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.layer.len(), 1);
        assert_eq!(result.layer[0][0].name.as_ref(), "Tab");
        assert_eq!(result.layer[0][0].rdev_key, Some(Key::Tab));
        assert_eq!(result.layer[0][1].name.as_ref(), "P");
        assert_eq!(result.layer[0][1].rdev_key, Some(Key::KeyP));
    }

    #[test]
    fn test_parser_invalid_sequence() {
        // Missing the leading ":" -> | Q | -
        let tokens = vec![
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
            },
            t_name("Q"),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
            },
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_err());
    }

    #[test]
    fn test_parser_missing_split() {
        // Missing the pipe between names: :| A B | -
        let tokens = vec![
            Token {
                token_type: TokenType::LineHead,
                value: ":".into(),
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
            },
            t_name("A"),
            t_name("B"), // Error here: Parser expects Split (|) after Name
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
            },
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_err());
    }

    #[test]
    fn test_parser_with_attr() {
        // Input sequence for: :| Tab [$10] | 'P' | -
        let tokens = vec![
            Token {
                token_type: TokenType::LineHead,
                value: ":".into(),
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
            },
            t_name("Tab"),
            Token {
                token_type: TokenType::LBracket,
                value: "[".into(),
            },
            Token {
                token_type: TokenType::Number,
                value: "10".into(),
            },
            Token {
                token_type: TokenType::RBracket,
                value: "]".into(),
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
            },
            t_name("P"),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
            },
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.layer.len(), 1);
        assert_eq!(result.layer[0][0].name.as_ref(), "Tab");
        assert_eq!(result.layer[0][0].rdev_key, Some(Key::Tab));
        assert_eq!(result.layer[0][0].width, 10);
        assert_eq!(result.layer[0][1].name.as_ref(), "P");
        assert_eq!(result.layer[0][1].rdev_key, Some(Key::KeyP));
        assert_eq!(result.layer[0][1].width, 4);
    }

}
