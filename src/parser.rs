use crate::env::*;
use crate::error::*;
use crate::layout::*;
use crate::lexer::*;
use ratatui::style::Color;
use rdev::Key;
use std::sync::Arc;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn get_line(&self) -> usize {
        self.tokens
            .get(self.current)
            .map(|t| t.line)
            .unwrap_or_else(|| self.tokens.last().map(|t| t.line).unwrap_or(0))
    }

    fn peek(&self) -> Result<&Token, ParserError> {
        if self.current < self.tokens.len() {
            Ok(&self.tokens[self.current])
        } else {
            Err(ParserError::Err("EOF".to_string(), self.get_line()))
        }
    }

    fn advance(&mut self) -> Result<&Token, ParserError> {
        if self.current < self.tokens.len() {
            self.current += 1;
            Ok(&self.tokens[self.current - 1])
        } else {
            Err(ParserError::Err("EOF".to_string(), self.get_line()))
        }
    }

    fn consume(&mut self, ty: TokenType) -> Result<&Token, ParserError> {
        let t = self.peek()?;
        if t.token_type == ty {
            self.advance()
        } else {
            Err(ParserError::Err(
                format!("Expected {:?}, found {:?}", ty, t.token_type),
                self.get_line(),
            ))
        }
    }

    pub fn parse(&mut self, env: &mut Env) -> Result<Layout, ParserError> {
        let mut layer = Vec::new();

        while self.current < self.tokens.len() {
            match self.peek()?.token_type {
                TokenType::Ident => self.parse_assign(env)?,
                TokenType::LineHead => {
                    let row = self.parse_line(env)?;
                    layer.push(row);
                }
                _ => break,
            }
        }

        Ok(Layout { layer })
    }

    pub fn parse_assign(&mut self, env: &mut Env) -> Result<(), ParserError> {
        let ident = self.consume(TokenType::Ident)?;
        let name = ident.value.clone();
        self.consume(TokenType::Equal)?;
        let v = self.parse_value(env)?;
        env.insert(&name, v);
        Ok(())
    }

    fn parse_value(&mut self, env: &Env) -> Result<Value, ParserError> {
        match self.peek()?.token_type {
            TokenType::Number => {
                let num = self.consume(TokenType::Number)?;
                Ok(Value::Number(num.value.parse()?))
            }
            TokenType::Name => {
                let str = self.consume(TokenType::Name)?;
                Ok(Value::Str(str.value.clone().into()))
            }
            TokenType::At => {
                self.consume(TokenType::At)?;
                self.consume(TokenType::LParen)?;
                let r = self.consume(TokenType::Number)?.clone();
                self.consume(TokenType::Comma)?;
                let g = self.consume(TokenType::Number)?.clone();
                self.consume(TokenType::Comma)?;
                let b = self.consume(TokenType::Number)?.clone();
                self.consume(TokenType::RParen)?;
                Ok(Value::RGB(
                    r.value.parse()?,
                    g.value.parse()?,
                    b.value.parse()?,
                ))
            }
            TokenType::Ident => {
                let ident = self.consume(TokenType::Ident)?;
                let name = ident.value.clone();
                match env.get(name.as_str()) {
                    Some(v) => Ok(v.clone()),
                    None => Err(ParserError::Err(
                        format!("Unbounded Variable {:?}.", name),
                        self.get_line(),
                    )),
                }
            }
            _ => Err(ParserError::Err(
                "Expected Number, Identifier or RGB.".to_string(),
                self.get_line(),
            )),
        }
    }

    fn parse_line(&mut self, env: &Env) -> Result<Vec<Button>, ParserError> {
        let mut row = Vec::new();
        self.consume(TokenType::LineHead)?;
        self.consume(TokenType::Split)?;

        while self.current < self.tokens.len() && self.peek()?.token_type != TokenType::LineTail {
            let name_token = self.consume(TokenType::Name)?;
            let name_str = name_token.value.clone();
            let mut binds = vec![];
            let mut attr = Attr::default(&name_str);

            let r_key = if self.peek()?.token_type == TokenType::LBrace {
                self.consume(TokenType::LBrace)?;
                let spec_token = self.consume(TokenType::Name)?;
                let spec_key = spec_token.value.clone();
                self.consume(TokenType::RBrace)?;
                spec_key
            } else {
                name_str.clone()
            };

            binds.push((Arc::from(name_str.as_str()), get_rdev_key(&r_key)));

            while self.peek()?.token_type == TokenType::Comma {
                self.consume(TokenType::Comma)?;
                let name_token = self.consume(TokenType::Name)?;
                let name_str = name_token.value.clone();

                let r_key = if self.peek()?.token_type == TokenType::LBrace {
                    self.consume(TokenType::LBrace)?;
                    let spec_token = self.consume(TokenType::Name)?;
                    let spec_key = spec_token.value.clone();
                    self.consume(TokenType::RBrace)?;
                    spec_key
                } else {
                    name_str.clone()
                };

                binds.push((Arc::from(name_str.as_str()), get_rdev_key(&r_key)));
            }

            if self.peek()?.token_type == TokenType::LBracket {
                self.parse_attr(&mut attr, env)?;
            }

            row.push(Button { attr: attr, binds });

            self.consume(TokenType::Split)?;
        }

        self.consume(TokenType::LineTail)?;
        Ok(row)
    }

    fn parse_attr(&mut self, attr: &mut Attr, env: &Env) -> Result<(), ParserError> {
        // [width, border_color, highlight, alignment]
        self.consume(TokenType::LBracket)?;

        let mut pos = 0;

        while self.peek()?.token_type != TokenType::RBracket {
            let t_type = self.peek()?.token_type;

            if t_type == TokenType::Comma {
                self.consume(TokenType::Comma)?;
                pos += 1;
                continue;
            }

            match pos {
                0 => {
                    // width
                    if let Value::Number(w) = self.parse_value(env)? {
                        attr.width = w;
                    } else {
                        return Err(ParserError::Err("Width must be a number".into(), self.get_line()));
                    }
                }
                1 => {
                    // border_color
                    if let Value::RGB(r, g, b) = self.parse_value(env)? {
                        attr.border_color = Some(Color::Rgb(r, g, b));
                    } else {
                        return Err(ParserError::Err("Border color must be RGB".into(), self.get_line()));
                    }
                }
                2 => {
                    // highlight
                    if let Value::RGB(r, g, b) = self.parse_value(env)? {
                        attr.highlight = Some(Color::Rgb(r, g, b));
                    } else {
                        return Err(ParserError::Err("Highlight must be RGB".into(), self.get_line()));
                    }
                }
                3 => {
                    // alignment
                    if let Value::Str(v) = self.parse_value(env)? {
                        attr.alignment = Some(v);
                    } else {
                        return Err(ParserError::Err("Alignment must be Name".into(), self.get_line()));
                    }
                }

                _ => {
                    self.advance()?;
                }
            }

            if self.peek()?.token_type == TokenType::Comma {
                self.consume(TokenType::Comma)?;
                pos += 1;
            }
        }

        self.consume(TokenType::RBracket)?;
        Ok(())
    }
}

fn get_rdev_key(name: &str) -> Option<Key> {
    match name.to_lowercase().as_str() {
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
        "a" => Some(Key::KeyA),
        "s" => Some(Key::KeyS),
        "d" => Some(Key::KeyD),
        "f" => Some(Key::KeyF),
        "g" => Some(Key::KeyG),
        "h" => Some(Key::KeyH),
        "j" => Some(Key::KeyJ),
        "k" => Some(Key::KeyK),
        "l" => Some(Key::KeyL),
        "z" => Some(Key::KeyZ),
        "x" => Some(Key::KeyX),
        "c" => Some(Key::KeyC),
        "v" => Some(Key::KeyV),
        "b" => Some(Key::KeyB),
        "n" => Some(Key::KeyN),
        "m" => Some(Key::KeyM),

        "lshift" | "shift" => Some(Key::ShiftLeft),
        "rshift" => Some(Key::ShiftRight),
        "esc" | "escape" => Some(Key::Escape),
        "ctrl" | "lctrl" => Some(Key::ControlLeft),
        "rctrl" => Some(Key::ControlRight),
        "alt" | "lalt" => Some(Key::Alt),
        "ralt" | "altgr" => Some(Key::AltGr),
        "space" => Some(Key::Space),
        "enter" | "return" => Some(Key::Return),
        "caps" | "capslock" => Some(Key::CapsLock),
        "back" | "backspace" => Some(Key::Backspace),
        "tab" => Some(Key::Tab),

        "up" => Some(Key::UpArrow),
        "down" => Some(Key::DownArrow),
        "left" => Some(Key::LeftArrow),
        "right" => Some(Key::RightArrow),

        "ins" | "insert" => Some(Key::Insert),
        "del" | "delete" => Some(Key::Delete),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pgup" | "pageup" => Some(Key::PageUp),
        "pgdn" | "pagedown" => Some(Key::PageDown),

        "`" | "backquote" => Some(Key::BackQuote),
        "-" | "minus" => Some(Key::Minus),
        "=" | "equal" => Some(Key::Equal),
        "[" | "leftbracket" => Some(Key::LeftBracket),
        "]" | "rightbracket" => Some(Key::RightBracket),
        ";" | "semicolon" => Some(Key::SemiColon),
        "'" | "quote" => Some(Key::Quote),
        "\\" | "backslash" => Some(Key::BackSlash),
        "," | "comma" => Some(Key::Comma),
        "." | "dot" => Some(Key::Dot),
        "/" | "slash" => Some(Key::Slash),

        "f1" => Some(Key::F1),
        "f2" => Some(Key::F2),
        "f3" => Some(Key::F3),
        "f4" => Some(Key::F4),
        "f5" => Some(Key::F5),
        "f6" => Some(Key::F6),
        "f7" => Some(Key::F7),
        "f8" => Some(Key::F8),
        "f9" => Some(Key::F9),
        "f10" => Some(Key::F10),
        "f11" => Some(Key::F11),
        "f12" => Some(Key::F12),

        "win" | "meta" | "command" => Some(Key::MetaLeft),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdev::Key;

    // Helper to create a Name token
    fn t_name(val: &str, line: usize) -> Token {
        Token {
            token_type: TokenType::Name,
            value: val.to_string(),
            line,
        }
    }

    #[test]
    fn test_parser_success() {
        // Input sequence for: :| Tab | 'P' | -
        let tokens = vec![
            Token {
                token_type: TokenType::LineHead,
                value: ":".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            t_name("Tab", 1),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            t_name("P", 1),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
                line: 1,
            },
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse(&mut Env::new()).unwrap();

        assert_eq!(result.layer.len(), 1);
        assert_eq!(result.layer[0][0].binds[0].0.as_ref(), "Tab");
        assert_eq!(result.layer[0][0].binds[0].1, Some(Key::Tab));
        assert_eq!(result.layer[0][1].binds[0].0.as_ref(), "P");
        assert_eq!(result.layer[0][1].binds[0].1, Some(Key::KeyP));
    }

    #[test]
    fn test_parser_missing_split() {
        // Missing the pipe between names: :| A B | -
        let tokens = vec![
            Token {
                token_type: TokenType::LineHead,
                value: ":".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            t_name("A", 1),
            t_name("B", 1), // Error here: Parser expects Split (|) after Name
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
                line: 1,
            },
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse(&mut Env::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_with_attr() {
        // Input sequence for: :| Tab [$10] | 'P' | -
        let tokens = vec![
            Token {
                token_type: TokenType::LineHead,
                value: ":".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            t_name("Tab", 1),
            Token {
                token_type: TokenType::LBracket,
                value: "[".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Number,
                value: "10".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::RBracket,
                value: "]".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            t_name("P", 1),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
                line: 1,
            },
        ];

        let mut parser = Parser::new(tokens);

        let result = parser.parse(&mut Env::new()).unwrap();
        assert_eq!(result.layer.len(), 1);
        assert_eq!(result.layer[0][0].binds[0].0.as_ref(), "Tab");
        assert_eq!(result.layer[0][0].binds[0].1, Some(Key::Tab));
        assert_eq!(result.layer[0][0].attr.width, 10);
        assert_eq!(result.layer[0][1].binds[0].0.as_ref(), "P");
        assert_eq!(result.layer[0][1].binds[0].1, Some(Key::KeyP));
        assert_eq!(result.layer[0][1].attr.width, 4);
    }

    #[test]
    fn test_omit_attr() {
        // Input sequence for: :| Tab [$10, , @($1, $1, $1)] | -
        let tokens = vec![
            Token {
                token_type: TokenType::LineHead,
                value: ":".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            t_name("Tab", 1),
            Token {
                token_type: TokenType::LBracket,
                value: "[".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Number,
                value: "10".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Comma,
                value: ",".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Comma,
                value: ",".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::At,
                value: "@".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::LParen,
                value: "(".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Number,
                value: "1".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Comma,
                value: ",".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Number,
                value: "1".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Comma,
                value: ",".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Number,
                value: "1".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::RParen,
                value: ")".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::RBracket,
                value: "]".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
                line: 1,
            },
        ];

        let mut parser = Parser::new(tokens);

        let result = parser.parse(&mut Env::new()).unwrap();
        assert_eq!(result.layer.len(), 1);
        assert_eq!(result.layer[0][0].binds[0].0.as_ref(), "Tab");
        assert_eq!(result.layer[0][0].binds[0].1, Some(Key::Tab));
        assert_eq!(result.layer[0][0].attr.width, 10);
        assert_eq!(result.layer[0][0].attr.highlight, Some(Color::Rgb(1, 1, 1)));
    }

    #[test]
    fn test_multi_binds() {
        // Input sequence for: :| A, C, D| B | -
        let tokens = vec![
            Token {
                token_type: TokenType::LineHead,
                value: ":".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            t_name("A", 1),
            Token {
                token_type: TokenType::Comma,
                value: ",".into(),
                line: 1,
            },
            t_name("C", 1),
            Token {
                token_type: TokenType::Comma,
                value: ",".into(),
                line: 1,
            },
            t_name("D", 1),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            t_name("B", 1),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
                line: 1,
            },
        ];

        let mut parser = Parser::new(tokens);

        let result = parser.parse(&mut Env::new()).unwrap();
        assert_eq!(result.layer.len(), 1);
        assert_eq!(
            result.layer[0][0].binds,
            [
                (Arc::from("A"), Some(Key::KeyA)),
                (Arc::from("C"), Some(Key::KeyC)),
                (Arc::from("D"), Some(Key::KeyD)),
            ]
        );
        assert_eq!(result.layer[0][1].binds[0].0.as_ref(), "B");
        assert_eq!(result.layer[0][1].binds[0].1, Some(Key::KeyB));
        assert_eq!(result.layer[0][1].attr.width, 4);
    }

    #[test]
    fn test_declarations_with_layout() {
        let mut env = Env::new();

        // #id = $10
        // #color = @($1, $2, $3)
        // #str = "hi"
        // :| A, C, D | B |-
        let tokens = vec![
            Token {
                token_type: TokenType::Ident,
                value: "id".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Equal,
                value: "=".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Number,
                value: "10".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Ident,
                value: "color".into(),
                line: 2,
            },
            Token {
                token_type: TokenType::Equal,
                value: "=".into(),
                line: 2,
            },
            Token {
                token_type: TokenType::At,
                value: "@".into(),
                line: 2,
            },
            Token {
                token_type: TokenType::LParen,
                value: "(".into(),
                line: 2,
            },
            Token {
                token_type: TokenType::Number,
                value: "1".into(),
                line: 2,
            },
            Token {
                token_type: TokenType::Comma,
                value: ",".into(),
                line: 2,
            },
            Token {
                token_type: TokenType::Number,
                value: "2".into(),
                line: 2,
            },
            Token {
                token_type: TokenType::Comma,
                value: ",".into(),
                line: 2,
            },
            Token {
                token_type: TokenType::Number,
                value: "3".into(),
                line: 2,
            },
            Token {
                token_type: TokenType::RParen,
                value: ")".into(),
                line: 2,
            },
            Token {
                token_type: TokenType::Ident,
                value: "str".into(),
                line: 3,
            },
            Token {
                token_type: TokenType::Equal,
                value: "=".into(),
                line: 3,
            },
            Token {
                token_type: TokenType::Name,
                value: "hi".into(),
                line: 3,
            },
            Token {
                token_type: TokenType::LineHead,
                value: ":".into(),
                line: 4,
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 4,
            },
            t_name("A", 4),
            Token {
                token_type: TokenType::Comma,
                value: ",".into(),
                line: 4,
            },
            t_name("C", 4),
            Token {
                token_type: TokenType::Comma,
                value: ",".into(),
                line: 4,
            },
            t_name("D", 4),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 4,
            },
            t_name("B", 4),
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 4,
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
                line: 4,
            },
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse(&mut env).expect("Parsing failed");

        match env.get("id") {
            Some(Value::Number(n)) => assert_eq!(*n, 10),
            _ => panic!("Variable 'id' not found or wrong type"),
        }

        match env.get("color") {
            Some(Value::RGB(r, g, b)) => {
                assert_eq!(*r, 1);
                assert_eq!(*g, 2);
                assert_eq!(*b, 3);
            }
            _ => panic!("Variable 'color' not found or wrong type"),
        }

        match env.get("str") {
            Some(Value::Str(v)) => assert_eq!(*v, "hi".into()),
            _ => panic!("Variable 'str' not found or wrong type"),
        }

        assert_eq!(result.layer.len(), 1);

        let button_1 = &result.layer[0][0];
        assert_eq!(
            button_1.binds,
            vec![
                (Arc::from("A"), Some(Key::KeyA)),
                (Arc::from("C"), Some(Key::KeyC)),
                (Arc::from("D"), Some(Key::KeyD)),
            ]
        );

        let button_2 = &result.layer[0][1];
        assert_eq!(button_2.binds[0].0.as_ref(), "B");
        assert_eq!(button_2.binds[0].1, Some(Key::KeyB));
        assert_eq!(button_2.attr.width, 4);
    }

    #[test]
    fn test_customized_name() {
        // Input sequence for: :| ^ {"up"} | -
        let tokens = vec![
            Token {
                token_type: TokenType::LineHead,
                value: ":".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            t_name("^", 1),
            Token {
                token_type: TokenType::LBrace,
                value: "{".into(),
                line: 1,
            },
            t_name("up", 1),
            Token {
                token_type: TokenType::RBrace,
                value: "}".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::Split,
                value: "|".into(),
                line: 1,
            },
            Token {
                token_type: TokenType::LineTail,
                value: "-".into(),
                line: 1,
            },
        ];

        let mut parser = Parser::new(tokens);

        let result = parser.parse(&mut Env::new()).unwrap();
        assert_eq!(result.layer.len(), 1);
        assert_eq!(
            result.layer[0][0].binds,
            [(Arc::from("^"), Some(Key::UpArrow)),]
        );
    }
}
