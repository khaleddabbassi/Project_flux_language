// src/ir.rs
use std::collections::HashMap;

pub type IP = usize;
pub type FuncTable = HashMap<String, IP>;

#[derive(Debug, Clone)]
pub enum IR {
    PushI(i64), PushF(f64), PushS(String), PushB(bool), PushNull,
    Load(String), Store(String),
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, Neq, Lt, Gt, Le, Ge,
    And, Or, Not,
    Jump(IP), JumpFalse(IP),
    Call(String, usize),
    Return,
    Print,
}

pub struct Codegen {
    pub code: Vec<IR>,
    pub functions: FuncTable,
}

impl Codegen {
    pub fn new() -> Self { Self { code: Vec::new(), functions: HashMap::new() } }

    fn patch(&mut self, pos: usize, target: IP) {
        match &mut self.code[pos] {
            IR::Jump(p) | IR::JumpFalse(p) => *p = target,
            _ => unreachable!(),
        }
    }

    pub fn emit(&mut self, ir: IR) -> usize {
        let pos = self.code.len();
        self.code.push(ir);
        pos
    }

    // ... (generate from AST - see full version below)
}