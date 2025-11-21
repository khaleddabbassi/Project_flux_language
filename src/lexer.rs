// src/lexer.rs
use crate::error::FluxError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Constant, Mutable, Assign, Yield, Course, Purpose, // ADDED: Yield
    When, Then, Persist, Differently, Otherwise,
    Iterate, Across, To, // ADDED: To
    And, Or, Not, Void,
    StringType, NumberType, FloatType, BooleanType,
    Int(i64), Float(f64), Str(String), Ident(String), True, False,
    Plus, Minus, Star, Slash, Percent, Power,
    EqEq, BangEq, Lt, Gt, LtEq, GtEq, Eq,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket, Semicolon, Comma,
    EOF, // REMOVED: DotDot
}

pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { input: source.as_bytes(), pos: 0 }
    }

    fn advance(&mut self) { self.pos += 1; }
    fn cur(&self) -> u8 { self.input.get(self.pos).copied().unwrap_or(0) }
    fn peek(&self) -> u8 { self.input.get(self.pos + 1).copied().unwrap_or(0) }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.cur() {
                b' ' | b'\t' | b'\n' | b'\r' => self.advance(),
                b'/' if self.peek() == b'/' => self.skip_line_comment(),
                _ => break,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while self.pos < self.input.len() && self.cur() != b'\n' {
            self.advance();
        }
    }

    pub fn lex(mut self) -> Result<Vec<Token>, FluxError> {
        let mut tokens = Vec::with_capacity(self.input.len() / 6);
        
        while self.pos < self.input.len() {
            self.skip_whitespace();
            if self.pos >= self.input.len() { break; }

            match self.cur() {
                b'0'..=b'9' => {
                    let start = self.pos;
                    while matches!(self.cur(), b'0'..=b'9') { 
                        self.advance(); 
                    }
                    
                    if self.cur() == b'.' && matches!(self.peek(), b'0'..=b'9') {
                        self.advance();
                        while matches!(self.cur(), b'0'..=b'9') { 
                            self.advance(); 
                        }
                    }
                    
                    let s = std::str::from_utf8(&self.input[start..self.pos])
                        .map_err(|e| FluxError::Lex(format!("Invalid UTF-8: {}", e)))?;
                    
                    if s.contains('.') {
                        let f = s.parse().map_err(|_| FluxError::Lex(format!("Invalid float: {}", s)))?;
                        tokens.push(Token::Float(f));
                    } else {
                        let i = s.parse().map_err(|_| FluxError::Lex(format!("Invalid integer: {}", s)))?;
                        tokens.push(Token::Int(i));
                    }
                }
                b'"' => {
                    self.advance();
                    let start = self.pos;
                    while self.pos < self.input.len() && self.cur() != b'"' {
                        self.advance();
                    }
                    let s = String::from_utf8_lossy(&self.input[start..self.pos]).to_string();
                    tokens.push(Token::Str(s));
                    if self.cur() == b'"' {
                        self.advance();
                    }
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                    let start = self.pos;
                    while self.pos < self.input.len() && matches!(self.cur(), b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') {
                        self.advance();
                    }
                    let word = std::str::from_utf8(&self.input[start..self.pos])
                        .map_err(|e| FluxError::Lex(format!("Invalid UTF-8: {}", e)))?;
                    let token = match word {
                        "constant" => Token::Constant,
                        "mutable" => Token::Mutable,
                        "assign" => Token::Assign,
                        "yield" => Token::Yield,        // ADDED
                        "course" => Token::Course,
                        "purpose" => Token::Purpose,
                        "when" => Token::When,
                        "then" => Token::Then,
                        "persist" => Token::Persist,
                        "differently" => Token::Differently,
                        "otherwise" => Token::Otherwise,
                        "iterate" => Token::Iterate,
                        "across" => Token::Across,
                        "to" => Token::To,              // ADDED
                        "and" => Token::And,
                        "or" => Token::Or,
                        "not" => Token::Not,
                        "true" => Token::True,
                        "false" => Token::False,
                        "string" => Token::StringType,
                        "number" => Token::NumberType,
                        "float" => Token::FloatType,
                        "boolean" => Token::BooleanType,
                        "void" => Token::Void,
                        _ => Token::Ident(word.to_string()),
                    };
                    tokens.push(token);
                }
                b'+' => { tokens.push(Token::Plus); self.advance(); }
                b'-' => { tokens.push(Token::Minus); self.advance(); }
                b'*' => { 
                    self.advance(); 
                    if self.cur() == b'*' { 
                        self.advance(); 
                        tokens.push(Token::Power); 
                    } else { 
                        tokens.push(Token::Star); 
                    } 
                }
                b'/' => { tokens.push(Token::Slash); self.advance(); }
                b'%' => { tokens.push(Token::Percent); self.advance(); }
                b'=' => { 
                    self.advance(); 
                    if self.cur() == b'=' { 
                        self.advance(); 
                        tokens.push(Token::EqEq); 
                    } else { 
                        tokens.push(Token::Eq); 
                    } 
                }
                b'!' => { 
                    self.advance(); 
                    if self.cur() == b'=' { 
                        self.advance(); 
                        tokens.push(Token::BangEq); 
                    } else { 
                        tokens.push(Token::Not); 
                    } 
                }
                b'<' => { 
                    self.advance(); 
                    if self.cur() == b'=' { 
                        self.advance(); 
                        tokens.push(Token::LtEq); 
                    } else { 
                        tokens.push(Token::Lt); 
                    } 
                }
                b'>' => { 
                    self.advance(); 
                    if self.cur() == b'=' { 
                        self.advance(); 
                        tokens.push(Token::GtEq); 
                    } else { 
                        tokens.push(Token::Gt); 
                    } 
                }
                b'[' => { tokens.push(Token::LBracket); self.advance(); }
                b']' => { tokens.push(Token::RBracket); self.advance(); }
                b'(' => { tokens.push(Token::LParen); self.advance(); }
                b')' => { tokens.push(Token::RParen); self.advance(); }
                b'{' => { tokens.push(Token::LBrace); self.advance(); }
                b'}' => { tokens.push(Token::RBrace); self.advance(); }
                b';' => { tokens.push(Token::Semicolon); self.advance(); }
                b',' => { tokens.push(Token::Comma); self.advance(); }
                b'.' => { 
                    // Single dot is invalid now that we removed DotDot
                    return Err(FluxError::Lex("Invalid token: single '.'".to_string()));
                }
                ch => {
                    let ch = ch as char;
                    self.advance();
                    return Err(FluxError::Lex(format!("Unexpected character: '{}'", ch)));
                }
            }
        }
        tokens.push(Token::EOF);
        Ok(tokens)
    }
}