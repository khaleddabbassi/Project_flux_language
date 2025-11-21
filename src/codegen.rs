// src/codegen.rs
use crate::ast::*;
use std::collections::HashMap;

pub type IP = usize;
pub type FuncTable = HashMap<String, IP>;

#[derive(Debug, Clone)]
pub enum IR {
    PushI(i64), PushF(f64), PushS(String), PushB(bool), PushNull,
    Load(String), Store(String),
    Add, Sub, Mul, Div, Mod, Power,
    Eq, Neq, Lt, Gt, Le, Ge, And, Or, Not,
    Jump(IP), JumpFalse(IP),
    Call(String, usize), Return,
    // List operations
    MakeList(usize), GetIndex, SetIndex, ListLen,
}

pub struct Codegen {
    pub code: Vec<IR>,
    pub functions: FuncTable,
}

impl Codegen {
    pub fn new() -> Self { 
        Self { 
            code: Vec::with_capacity(8192), 
            functions: HashMap::new() 
        } 
    }

    fn emit(&mut self, op: IR) -> usize { 
        let p = self.code.len(); 
        self.code.push(op); 
        p 
    }
    
    fn patch(&mut self, pos: usize, target: IP) {
        match &mut self.code[pos] {
            IR::Jump(t) | IR::JumpFalse(t) => *t = target,
            _ => {}
        }
    }

    // In Codegen::compile
	pub fn compile(&mut self, stmts: &[Stmt]) {
		// 1. New: Reserve a spot for the initial jump to the main execution code.
		// The target is temporarily set to 0.
		let main_jump_pos = self.emit(IR::Jump(0));

		// STEP 1: Compile ALL function definitions FIRST (Code will be placed before the jump target)
		for s in stmts {
			if let Stmt::Course { name, params, body } | Stmt::Purpose { name, params, body } = s {
				let entry = self.code.len();
				self.functions.insert(name.clone(), entry);
				
				// Function prologue: store parameters
				for p in params.iter().rev() { 
					self.emit(IR::Store(p.clone())); 
				}
				
				// Function body
				for stmt in body { 
					self.stmt(stmt); 
				}
				
				// Function epilogue: ensure return
				self.emit(IR::PushNull);
				self.emit(IR::Return);
			}
		}
		
		// 2. New: Get the actual IP for the start of the global statements.
		let main_entry_ip = self.code.len();
		
		// 3. New: Patch the initial jump to point to the start of the global statements.
		self.patch(main_jump_pos, main_entry_ip);

		// STEP 2: Compile global statements ONLY
		for s in stmts {
			match s {
				Stmt::Const { .. } | Stmt::Mutable { .. } | Stmt::Assign { .. } | 
				Stmt::Expr(_) | Stmt::Iterate { .. } | Stmt::Persist { .. } | 
				Stmt::When { .. } | Stmt::Block(_) => {
					self.stmt(s);
				}
				Stmt::Course { .. } | Stmt::Purpose { .. } => {
					// Already compiled in step 1, and now execution will jump over them.
				}
				_ => {}
			}
		}
		
		// Add final return for execution
		self.emit(IR::PushNull);
		self.emit(IR::Return);
	}

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Const { name, value } | Stmt::Mutable { name, init: Some(value), .. } => {
                self.expr(value);
                self.emit(IR::Store(name.clone()));
            }
            Stmt::Mutable { name, init: None, .. } => {
                self.emit(IR::PushNull);
                self.emit(IR::Store(name.clone()));
            }
            Stmt::Assign { name, value } => {
                // Handle list assignment: name[index] = value
                if let Expr::Index { target, index, value: assignment_value } = value {
                    if let Expr::Ident(var_name) = &**target {
                        // Load the list, index, and value
                        self.emit(IR::Load(var_name.clone()));
                        self.expr(&index);
                        if let Some(assignment_value) = assignment_value {
                            self.expr(&assignment_value);
                        } else {
                            self.emit(IR::PushNull);
                        }
                        self.emit(IR::SetIndex);
                        self.emit(IR::Store(var_name.clone())); // Store back the modified list
                        return;
                    }
                }
                // Regular assignment
                self.expr(value);
                self.emit(IR::Store(name.clone()));
            }
            Stmt::Expr(e) => { 
                self.expr(e);
            }
            Stmt::Return(Some(e)) => { self.expr(e); self.emit(IR::Return); }
            Stmt::Return(None) => { self.emit(IR::PushNull); self.emit(IR::Return); }
            Stmt::Persist { cond, body } => {
                let start = self.code.len();
                self.expr(cond);
                let jf = self.emit(IR::JumpFalse(0));
                for b in body { self.stmt(b); }
                self.emit(IR::Jump(start));
                self.patch(jf, self.code.len());
            }
            Stmt::When { cond, then, elifs, otherwise } => {
                self.expr(cond);
                let mut exit_jumps = vec![];
                let mut cond_jumps = vec![self.emit(IR::JumpFalse(0))];

                for s in then { self.stmt(s); }
                exit_jumps.push(self.emit(IR::Jump(0)));
                self.patch(cond_jumps[0], self.code.len());

                for (c, b) in elifs {
                    self.expr(c);
                    let j = self.emit(IR::JumpFalse(0));
                    cond_jumps.push(j);
                    for s in b { self.stmt(s); }
                    exit_jumps.push(self.emit(IR::Jump(0)));
                }
                for s in otherwise { self.stmt(s); }

                let end = self.code.len();
                for j in exit_jumps { self.patch(j, end); }
            }
            Stmt::Iterate { var, iterable, body } => {
                // Check if this is a range iteration (1 to 10)
                if let Expr::Binary { left, op: crate::lexer::Token::To, right } = iterable {
                    // Range iteration: variable i = start
                    self.expr(left);
                    self.emit(IR::Store(var.clone()));
                    
                    let loop_start = self.code.len();
                    
                    // Condition: i <= end
                    self.emit(IR::Load(var.clone()));
                    self.expr(right);
                    self.emit(IR::Le);
                    let jf = self.emit(IR::JumpFalse(0));
                    
                    // Loop body
                    for b in body { self.stmt(b); }
                    
                    // Increment: i = i + 1
                    self.emit(IR::Load(var.clone()));
                    self.emit(IR::PushI(1));
                    self.emit(IR::Add);
                    self.emit(IR::Store(var.clone()));
                    
                    // Jump back
                    self.emit(IR::Jump(loop_start));
                    self.patch(jf, self.code.len());
                } else {
                    // Iterate over list or other iterable
                    self.expr(iterable);
                    self.emit(IR::Store("_iter_list".to_string()));
                    self.emit(IR::PushI(0));
                    self.emit(IR::Store("_iter_index".to_string()));
                    
                    let loop_start = self.code.len();
                    self.emit(IR::Load("_iter_index".to_string()));
                    self.emit(IR::Load("_iter_list".to_string()));
                    self.emit(IR::ListLen);
                    self.emit(IR::Lt);
                    let jf = self.emit(IR::JumpFalse(0));
                    
                    // Get current element
                    self.emit(IR::Load("_iter_list".to_string()));
                    self.emit(IR::Load("_iter_index".to_string()));
                    self.emit(IR::GetIndex);
                    self.emit(IR::Store(var.clone()));
                    
                    // Loop body
                    for b in body { self.stmt(b); }
                    
                    // Increment index
                    self.emit(IR::Load("_iter_index".to_string()));
                    self.emit(IR::PushI(1));
                    self.emit(IR::Add);
                    self.emit(IR::Store("_iter_index".to_string()));
                    
                    self.emit(IR::Jump(loop_start));
                    self.patch(jf, self.code.len());
                }
            }
            Stmt::Course { .. } | Stmt::Purpose { .. } => {
                // These are handled separately in compile()
            }
            Stmt::Block(body) => {
                for stmt in body {
                    self.stmt(stmt);
                }
            }
        }
    }

    fn expr(&mut self, e: &Expr) {
        match e {
            Expr::Int(i) => { self.emit(IR::PushI(*i)); }
            Expr::Float(f) => { self.emit(IR::PushF(*f)); }
            Expr::Str(s) => { self.emit(IR::PushS(s.clone())); }
            Expr::Bool(b) => { self.emit(IR::PushB(*b)); }
            Expr::List(elements) => {
                for elem in elements {
                    self.expr(elem);
                }
                self.emit(IR::MakeList(elements.len()));
            }
            Expr::Ident(n) => { self.emit(IR::Load(n.clone())); }
            Expr::Call { callee, args } => {
                for a in args { self.expr(a); }
                self.emit(IR::Call(callee.clone(), args.len()));
            }
            Expr::Index { target, index, value } => {
                self.expr(target);
                self.expr(index);
                if let Some(assignment_value) = value {
                    self.expr(&assignment_value);
                    self.emit(IR::SetIndex);
                } else {
                    self.emit(IR::GetIndex);
                }
            }
            Expr::Binary { left, op, right } => {
                self.expr(left);
                self.expr(right);
                match op {
                    crate::lexer::Token::Plus => { self.emit(IR::Add); }
                    crate::lexer::Token::Minus => { self.emit(IR::Sub); }
                    crate::lexer::Token::Star => { self.emit(IR::Mul); }
                    crate::lexer::Token::Slash => { self.emit(IR::Div); }
                    crate::lexer::Token::Percent => { self.emit(IR::Mod); }
                    crate::lexer::Token::Power => { self.emit(IR::Power); }
                    crate::lexer::Token::EqEq => { self.emit(IR::Eq); }
                    crate::lexer::Token::BangEq => { self.emit(IR::Neq); }
                    crate::lexer::Token::Lt => { self.emit(IR::Lt); }
                    crate::lexer::Token::Gt => { self.emit(IR::Gt); }
                    crate::lexer::Token::LtEq => { self.emit(IR::Le); }
                    crate::lexer::Token::GtEq => { self.emit(IR::Ge); }
                    crate::lexer::Token::And => { self.emit(IR::And); }
                    crate::lexer::Token::Or => { self.emit(IR::Or); }
                    crate::lexer::Token::To => { 
                        // 'to' operator used in ranges - handled in iterate loops
                        self.emit(IR::Le);
                    }
                    _ => {}
                }
            }
            Expr::Unary { op: crate::lexer::Token::Minus, expr } => {
                self.expr(expr);
                self.emit(IR::PushI(-1));
                self.emit(IR::Mul);
            }
            Expr::Unary { op: crate::lexer::Token::Not, expr } => {
                self.expr(expr);
                self.emit(IR::Not);
            }
            _ => {}
        }
    }
}