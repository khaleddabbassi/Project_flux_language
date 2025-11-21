// src/parser.rs
use crate::ast::*;
use crate::error::FluxError;

type PResult<T> = Result<T, FluxError>;

pub struct Parser {
    tokens: Vec<crate::lexer::Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<crate::lexer::Token>) -> Self { 
        Self { tokens, pos: 0 } 
    }

    fn cur(&self) -> &crate::lexer::Token { 
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &crate::lexer::Token::EOF
        }
    }

    fn advance(&mut self) -> &crate::lexer::Token { 
        if self.pos < self.tokens.len() {
            self.pos += 1; 
        }
        &self.tokens[self.pos - 1] 
    }

    fn eat(&mut self, expected: crate::lexer::Token) -> PResult<()> {
        if std::mem::discriminant(self.cur()) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(FluxError::Parse(format!("Expected {:?}, found {:?} at position {}", expected, self.cur(), self.pos)))
        }
    }

    pub fn parse(&mut self) -> PResult<Vec<Stmt>> {
        let mut stmts = vec![];
        while !matches!(self.cur(), crate::lexer::Token::EOF) {
            stmts.push(self.stmt()?);
        }
        Ok(stmts)
    }

    fn stmt(&mut self) -> PResult<Stmt> {
        match self.cur() {
            crate::lexer::Token::Constant => self.const_decl(),
            crate::lexer::Token::Mutable => self.mutable_decl(),
            crate::lexer::Token::Assign => self.assign(),
            crate::lexer::Token::Yield => self.yield_stmt(),
            crate::lexer::Token::Course => self.course(),
            crate::lexer::Token::Purpose => self.purpose(),
            crate::lexer::Token::Persist => self.persist(),
            crate::lexer::Token::When => self.when(),
            crate::lexer::Token::Iterate => self.iterate_loop(),
            crate::lexer::Token::LBrace => {
                let block = self.block()?;
                Ok(Stmt::Block(block))
            }
            _ => {
                if self.is_assignment_target() {
                    self.assignment_stmt()
                } else {
                    let expr = self.expr()?;
                    self.eat(crate::lexer::Token::Semicolon)?;
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    fn is_assignment_target(&self) -> bool {
        let mut pos = self.pos;
        
        if !matches!(self.tokens.get(pos), Some(crate::lexer::Token::Ident(_))) {
            return false;
        }
        pos += 1;
        
        while matches!(self.tokens.get(pos), Some(crate::lexer::Token::LBracket)) {
            pos += 1;
            if !matches!(self.tokens.get(pos), Some(crate::lexer::Token::Int(_) | crate::lexer::Token::Ident(_))) {
                return false;
            }
            pos += 1;
            if !matches!(self.tokens.get(pos), Some(crate::lexer::Token::RBracket)) {
                return false;
            }
            pos += 1;
        }
        
        matches!(self.tokens.get(pos), Some(crate::lexer::Token::Eq))
    }

    fn assignment_stmt(&mut self) -> PResult<Stmt> {
        let target = self.expr()?;
        self.eat(crate::lexer::Token::Eq)?;
        let value = self.expr()?;
        self.eat(crate::lexer::Token::Semicolon)?;
        
        match target {
            Expr::Ident(name) => Ok(Stmt::Assign { name, value }),
            Expr::Index { target, index, value: _ } => {
                if let Expr::Ident(var_name) = *target {
                    let var_name_clone = var_name.clone();
                    Ok(Stmt::Assign { 
                        name: var_name, 
                        value: Expr::Index {
                            target: Box::new(Expr::Ident(var_name_clone)),
                            index,
                            value: Some(Box::new(value)),
                        }
                    })
                } else {
                    Err(FluxError::Parse("Invalid assignment target".to_string()))
                }
            }
            _ => Err(FluxError::Parse("Invalid assignment target".to_string()))
        }
    }

    fn const_decl(&mut self) -> PResult<Stmt> {
        self.eat(crate::lexer::Token::Constant)?; 
        let name = self.ident()?;
        let value = if matches!(self.cur(), crate::lexer::Token::Eq) {
            self.advance();
            self.expr()?
        } else {
            Expr::Int(0)
        };
        self.eat(crate::lexer::Token::Semicolon)?;
        Ok(Stmt::Const { name, value })
    }

    fn mutable_decl(&mut self) -> PResult<Stmt> {
        self.eat(crate::lexer::Token::Mutable)?; 
        let name = self.ident()?;
        let init = if matches!(self.cur(), crate::lexer::Token::Eq) { 
            self.advance(); 
            Some(self.expr()?) 
        } else { 
            None 
        };
        self.eat(crate::lexer::Token::Semicolon)?;
        Ok(Stmt::Mutable { name, init })
    }

    fn assign(&mut self) -> PResult<Stmt> {
        self.eat(crate::lexer::Token::Assign)?; 
        let name = self.ident()?; 
        self.eat(crate::lexer::Token::Eq)?; 
        let value = self.expr()?; 
        self.eat(crate::lexer::Token::Semicolon)?;
        Ok(Stmt::Assign { name, value })
    }

    fn yield_stmt(&mut self) -> PResult<Stmt> {
        self.eat(crate::lexer::Token::Yield)?;
        let val = if !matches!(self.cur(), crate::lexer::Token::Semicolon) { 
            Some(self.expr()?) 
        } else { 
            None 
        };
        self.eat(crate::lexer::Token::Semicolon)?;
        Ok(Stmt::Return(val))
    }

    fn course(&mut self) -> PResult<Stmt> {
        self.eat(crate::lexer::Token::Course)?; 
        let name = self.ident()?; 
        self.eat(crate::lexer::Token::LParen)?; 
        let params = self.params()?; 
        self.eat(crate::lexer::Token::RParen)?; 
        let body = self.block()?;
        Ok(Stmt::Course { name, params, body })
    }

    fn purpose(&mut self) -> PResult<Stmt> {
		self.eat(crate::lexer::Token::Purpose)?; 
		let name = self.ident()?; 
		self.eat(crate::lexer::Token::LParen)?; 
		let params = self.params()?; 
		self.eat(crate::lexer::Token::RParen)?; 
		let body = self.block()?;
		Ok(Stmt::Purpose { name, params, body })  // CHANGED: Stmt::Purpose
	}

    fn persist(&mut self) -> PResult<Stmt> {
        self.eat(crate::lexer::Token::Persist)?; 
        let cond = self.expr()?; 
        let body = self.block()?;
        Ok(Stmt::Persist { cond, body })
    }

    fn when(&mut self) -> PResult<Stmt> {
        self.eat(crate::lexer::Token::When)?; 
        let cond = self.expr()?; 
        self.eat(crate::lexer::Token::Then)?; 
        let then = self.block()?;
        let mut elifs = vec![];
        while matches!(self.cur(), crate::lexer::Token::Differently) { 
            self.advance(); 
            let c = self.expr()?; 
            self.eat(crate::lexer::Token::Then)?; 
            elifs.push((c, self.block()?)); 
        }
        let otherwise = if matches!(self.cur(), crate::lexer::Token::Otherwise) { 
            self.advance(); 
            self.block()? 
        } else { 
            vec![] 
        };
        Ok(Stmt::When { cond, then, elifs, otherwise })
    }

    fn iterate_loop(&mut self) -> PResult<Stmt> {
        self.eat(crate::lexer::Token::Iterate)?;
        let var = self.ident()?;
        self.eat(crate::lexer::Token::Across)?;
        let iterable = self.expr()?;  // This can be a range (1 to 10) or list
        let body = self.block()?;
        Ok(Stmt::Iterate { var, iterable, body })
    }

    fn block(&mut self) -> PResult<Vec<Stmt>> {
        self.eat(crate::lexer::Token::LBrace)?; 
        let mut stmts = vec![]; 
        while !matches!(self.cur(), crate::lexer::Token::RBrace) { 
            stmts.push(self.stmt()?); 
        } 
        self.eat(crate::lexer::Token::RBrace)?; 
        Ok(stmts)
    }

    fn params(&mut self) -> PResult<Vec<String>> {
        let mut p = vec![];
        if matches!(self.cur(), crate::lexer::Token::RParen) { 
            return Ok(p); 
        }
        loop {
            let name = self.ident()?; 
            p.push(name);
            if !matches!(self.cur(), crate::lexer::Token::Comma) { break; } 
            self.advance();
        }
        Ok(p)
    }

    fn ident(&mut self) -> PResult<String> {
        if let crate::lexer::Token::Ident(s) = self.cur() { 
            let n = s.clone(); 
            self.advance(); 
            Ok(n) 
        } else { 
            Err(FluxError::Parse(format!("Expected identifier, found {:?}", self.cur())))
        }
    }

    fn expr(&mut self) -> PResult<Expr> { 
        self.prec(0) 
    }

    fn prec(&mut self, min: u8) -> PResult<Expr> {
        let mut left = self.index_expr()?;
        
        while let Some((l, r)) = self.bp(self.cur()) {
            if l < min { break; }
            let op = self.advance().clone();
            let right = self.prec(r)?;
            left = Expr::Binary { 
                left: Box::new(left), 
                op, 
                right: Box::new(right) 
            };
        }
        Ok(left)
    }

    fn index_expr(&mut self) -> PResult<Expr> {
        let mut expr = self.atom()?;
        
        while matches!(self.cur(), crate::lexer::Token::LBracket) {
            self.advance();
            let index = self.expr()?;
            self.eat(crate::lexer::Token::RBracket)?;
            expr = Expr::Index {
                target: Box::new(expr),
                index: Box::new(index),
                value: None,
            };
        }
        
        Ok(expr)
    }

    fn atom(&mut self) -> PResult<Expr> {
        match self.cur() {
            crate::lexer::Token::Int(i) => { 
                let v = *i; 
                self.advance(); 
                Ok(Expr::Int(v)) 
            }
            crate::lexer::Token::Float(f) => { 
                let v = *f; 
                self.advance(); 
                Ok(Expr::Float(v)) 
            }
            crate::lexer::Token::Str(s) => { 
                let v = s.clone(); 
                self.advance(); 
                Ok(Expr::Str(v)) 
            }
            crate::lexer::Token::True => { 
                self.advance(); 
                Ok(Expr::Bool(true)) 
            }
            crate::lexer::Token::False => { 
                self.advance(); 
                Ok(Expr::Bool(false)) 
            }
            crate::lexer::Token::LBracket => self.list(),
            crate::lexer::Token::Ident(name) => {
                let n = name.clone(); 
                self.advance();
                if matches!(self.cur(), crate::lexer::Token::LParen) {
                    self.eat(crate::lexer::Token::LParen)?;
                    let mut args = vec![];
                    if !matches!(self.cur(), crate::lexer::Token::RParen) {
                        args.push(self.expr()?);
                        while matches!(self.cur(), crate::lexer::Token::Comma) { 
                            self.advance(); 
                            args.push(self.expr()?); 
                        }
                    }
                    self.eat(crate::lexer::Token::RParen)?;
                    Ok(Expr::Call { callee: n, args })
                } else {
                    Ok(Expr::Ident(n))
                }
            }
            crate::lexer::Token::LParen => { 
                self.eat(crate::lexer::Token::LParen)?; 
                let e = self.expr()?; 
                self.eat(crate::lexer::Token::RParen)?; 
                Ok(e) 
            }
            crate::lexer::Token::Minus => { 
                self.advance(); 
                let e = self.prec(9)?; 
                Ok(Expr::Unary { 
                    op: crate::lexer::Token::Minus, 
                    expr: Box::new(e) 
                }) 
            }
            crate::lexer::Token::Not => { 
                self.advance(); 
                let e = self.prec(9)?; 
                Ok(Expr::Unary { 
                    op: crate::lexer::Token::Not, 
                    expr: Box::new(e) 
                }) 
            }
            _ => Err(FluxError::Parse(format!("Unexpected token in expression: {:?}", self.cur()))),
        }
    }

    fn list(&mut self) -> PResult<Expr> {
        self.eat(crate::lexer::Token::LBracket)?;
        let mut elements = vec![];
        if !matches!(self.cur(), crate::lexer::Token::RBracket) {
            elements.push(self.expr()?);
            while matches!(self.cur(), crate::lexer::Token::Comma) {
                self.advance();
                elements.push(self.expr()?);
            }
        }
        self.eat(crate::lexer::Token::RBracket)?;
        Ok(Expr::List(elements))
    }

    fn bp(&self, t: &crate::lexer::Token) -> Option<(u8, u8)> {
        Some(match t {
            crate::lexer::Token::Or => (1, 2),
            crate::lexer::Token::And => (3, 4),
            crate::lexer::Token::EqEq | crate::lexer::Token::BangEq => (5, 6),
            crate::lexer::Token::Lt | crate::lexer::Token::Gt | 
            crate::lexer::Token::LtEq | crate::lexer::Token::GtEq => (7, 8),
            crate::lexer::Token::Plus | crate::lexer::Token::Minus => (9, 10),
            crate::lexer::Token::Star | crate::lexer::Token::Slash | 
            crate::lexer::Token::Percent => (11, 12),
            crate::lexer::Token::Power => (13, 14),
            crate::lexer::Token::To => (15, 16),  // Ranges have high precedence
            _ => return None,
        })
    }
}