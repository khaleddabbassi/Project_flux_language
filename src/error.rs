// src/error.rs
#[derive(Debug)]
pub enum FluxError {
    Lex(String),
    Parse(String),
}

impl std::fmt::Display for FluxError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FluxError::Lex(msg) => write!(f, "Lexer Error: {}", msg),
            FluxError::Parse(msg) => write!(f, "Parser Error: {}", msg),
        }
    }
}

impl std::error::Error for FluxError {}