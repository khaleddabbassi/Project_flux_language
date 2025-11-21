use crate::ast::Expr;
use std::io::{self, Write};

#[derive(Debug)]
pub enum BuiltinFunction {
    Report,
    GetInput,
}

impl BuiltinFunction {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "report" => Some(BuiltinFunction::Report),
            "getInput" => Some(BuiltinFunction::GetInput),
            _ => None,
        }
    }

    pub fn execute(&self, args: &[Expr]) -> Result<(), String> {
        match self {
            BuiltinFunction::Report => {
                let output: Vec<String> = args.iter()
                    .map(|arg| format!("{:?}", arg))
                    .collect();
                println!("{}", output.join(" "));
                Ok(())
            }
            BuiltinFunction::GetInput => {
                print!("Enter {} values: ", args.len());
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                
                println!("Got input: {}", input.trim());
                Ok(())
            }
        }
    }
}