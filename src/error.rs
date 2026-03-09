use thiserror::Error;
use std::num::ParseIntError;

#[derive(Error, Debug)]
pub enum ParserError{
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error(" [line: {1}] {0}")]
    Err(String, usize),
    #[error("IO error: {0}")]
    ParseIntError(#[from] ParseIntError),
}

#[derive(Error, Debug)]
pub enum LexerError{
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum AppError{
    #[error("Usage: tvk <path>")]
    WrongUsage,
    #[error("Parser error:\n {0}")]
    ParserErr(#[from] ParserError),
    #[error("Lexer error: {0}")]
    LexerErr(#[from] LexerError),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}



