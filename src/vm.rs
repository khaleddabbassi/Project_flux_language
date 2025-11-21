// src/vm.rs
use crate::codegen::{IR, FuncTable};
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Int(i64), Float(f64), Str(String), Bool(bool), Null,
    List(Vec<Value>),
}

impl Value {
    fn truthy(&self) -> bool { !matches!(self, Value::Bool(false) | Value::Null) }
    fn as_f64(&self) -> f64 {
        match self {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => 0.0,
        }
    }
    fn as_int(&self) -> i64 {
        match self {
            Value::Int(i) => *i,
            Value::Float(f) => *f as i64,
            _ => 0,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(v) => write!(f, "{}", v),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::List(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
        }
    }
}

fn bin_arith<F1, F2>(a: Value, b: Value, iop: F1, fop: F2) -> Value 
where 
    F1: Fn(i64,i64)->i64, 
    F2: Fn(f64,f64)->f64 
{
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(iop(a, b)),
        (Value::Float(a), Value::Float(b)) => Value::Float(fop(a, b)),
        (Value::Int(a), Value::Float(b)) => Value::Float(fop(a as f64, b)),
        (Value::Float(a), Value::Int(b)) => Value::Float(fop(a, b as f64)),
        _ => Value::Null,
    }
}

impl std::ops::Add for Value { 
    type Output = Value; 
    fn add(self, rhs: Value) -> Value { 
        bin_arith(self, rhs, |a,b| a + b, |a,b| a + b) 
    } 
}

impl std::ops::Sub for Value { 
    type Output = Value; 
    fn sub(self, rhs: Value) -> Value { 
        bin_arith(self, rhs, |a,b| a - b, |a,b| a - b) 
    } 
}

impl std::ops::Mul for Value { 
    type Output = Value; 
    fn mul(self, rhs: Value) -> Value { 
        bin_arith(self, rhs, |a,b| a * b, |a,b| a * b) 
    } 
}

impl std::ops::Div for Value { 
    type Output = Value; 
    fn div(self, rhs: Value) -> Value { 
        bin_arith(self, rhs, |a,b| a / b, |a,b| a / b) 
    } 
}

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    call_stack: Vec<usize>,
}

impl VM {
    pub fn new() -> Self { 
        Self { 
            stack: Vec::with_capacity(1024), 
            globals: HashMap::new(), 
            call_stack: Vec::new() 
        } 
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::Null)
    }

    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        let mut result = Vec::with_capacity(n);
        for _ in 0..n {
            result.push(self.pop());
        }
        result.reverse();
        result
    }

    pub fn run(&mut self, code: &[IR], functions: &FuncTable) {
        let mut ip = 0;
        let mut steps = 0;
        let max_steps = 10_000;
        
        while ip < code.len() && steps < max_steps {
            steps += 1;
            
            match &code[ip] {
                IR::PushI(n) => self.stack.push(Value::Int(*n)),
                IR::PushF(n) => self.stack.push(Value::Float(*n)),
                IR::PushS(s) => self.stack.push(Value::Str(s.clone())),
                IR::PushB(b) => self.stack.push(Value::Bool(*b)),
                IR::PushNull => self.stack.push(Value::Null),
                IR::Load(name) => {
                    let v = self.globals.get(name).cloned().unwrap_or(Value::Null);
                    self.stack.push(v);
                }
                IR::Store(name) => {
                    let v = self.pop();
                    self.globals.insert(name.clone(), v);
                }
                IR::Add => { 
                    let b = self.pop();
                    let a = self.pop();
                    let result = a + b;
                    self.stack.push(result); 
                }
                IR::Sub => { 
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(a - b); 
                }
                IR::Mul => { 
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(a * b); 
                }
                IR::Div => { 
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(a / b); 
                }
                IR::Mod => {
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(if let (Value::Int(a), Value::Int(b)) = (a, b) { 
                        Value::Int(a % b) 
                    } else { 
                        Value::Null 
                    });
                }
                IR::Power => {
                    let b = self.pop().as_f64();
                    let a = self.pop().as_f64();
                    self.stack.push(Value::Float(a.powf(b)));
                }
                IR::Eq => { 
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(Value::Bool(a == b)); 
                }
                IR::Neq => { 
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(Value::Bool(a != b)); 
                }
                IR::Lt => { 
                    let right = self.pop();
                    let left = self.pop();
                    let result = left.as_f64() < right.as_f64();
                    self.stack.push(Value::Bool(result)); 
                }
                IR::Gt => { 
                    let right = self.pop();
                    let left = self.pop();
                    self.stack.push(Value::Bool(left.as_f64() > right.as_f64())); 
                }
                IR::Le => { 
                    let right = self.pop();
                    let left = self.pop();
                    let result = left.as_f64() <= right.as_f64();
                    self.stack.push(Value::Bool(result)); 
                }
                IR::Ge => { 
                    let right = self.pop();
                    let left = self.pop();
                    self.stack.push(Value::Bool(left.as_f64() >= right.as_f64())); 
                }
                IR::And => { 
                    let b = self.pop().truthy(); 
                    let a = self.pop().truthy(); 
                    self.stack.push(Value::Bool(a && b)); 
                }
                IR::Or => { 
                    let b = self.pop().truthy(); 
                    let a = self.pop().truthy(); 
                    self.stack.push(Value::Bool(a || b)); 
                }
                IR::Not => { 
                    let v = self.pop().truthy(); 
                    self.stack.push(Value::Bool(!v)); 
                }
                // *** JUMP FIXES APPLIED ***
                IR::Jump(t) => {
                    ip = *t;
                    continue; // Skip ip += 1
                }
                IR::JumpFalse(t) => { 
                    if !self.pop().truthy() { 
                        ip = *t; 
                        continue; // Skip ip += 1 if jump is taken
                    } 
                }
                // *** END JUMP FIXES ***
                IR::MakeList(size) => {
                    let elements = self.pop_n(*size);
                    self.stack.push(Value::List(elements));
                }
                IR::GetIndex => {
                    let index = self.pop().as_int() as usize;
                    if let Value::List(list) = self.pop() {
                        if index < list.len() {
                            self.stack.push(list[index].clone());
                        } else {
                            self.stack.push(Value::Null);
                        }
                    } else {
                        self.stack.push(Value::Null);
                    }
                }
                IR::SetIndex => {
                    let value = self.pop();
                    let index = self.pop().as_int() as usize;
                    if let Value::List(mut list) = self.pop() {
                        if index < list.len() {
                            list[index] = value;
                            self.stack.push(Value::List(list));
                        } else {
                            self.stack.push(Value::Null);
                        }
                    } else {
                        self.stack.push(Value::Null);
                    }
                }
                IR::ListLen => {
                    if let Value::List(list) = self.pop() {
                        self.stack.push(Value::Int(list.len() as i64));
                    } else {
                        self.stack.push(Value::Int(0));
                    }
                }
                IR::Call(name, argc) => {
                    if name == "getInput" {
                        print!("Input: ");
                        io::stdout().flush().unwrap();
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        
                        // NOTE: getInput handles its own stack pushing
                        for _ in 0..*argc {
                            self.pop(); // Pop arguments that were pushed before the call
                        }
                        self.stack.push(Value::Str(input.trim().to_string()));
                    } else if name == "report" {
                        let args = self.pop_n(*argc);
                        for arg in args {
                            print!("{} ", arg);
                        }
                        println!();
                        self.stack.push(Value::Null); // report returns null
                    } else if let Some(&target) = functions.get(name) {
                        self.call_stack.push(ip + 1);
                        ip = target;
                        continue;
                    } else {
                        self.pop_n(*argc);
                        self.stack.push(Value::Null);
                    }
                }
                IR::Return => {
                    if let Some(ret) = self.call_stack.pop() {
                        ip = ret;
                        continue;
                    } else {
                        break;
                    }
                }
            }
            ip += 1;
        }
        
        if steps >= max_steps {
            // Keep error logging for critical limits
            eprintln!("Execution stopped: maximum steps exceeded");
        }
    }
}