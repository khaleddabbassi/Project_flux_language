// src/main.rs
mod ast;
mod lexer;
mod parser;
mod codegen;
mod vm;
mod error;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <file.fl>", args.get(0).unwrap_or(&"flux".to_string()));
        eprintln!("Example: cargo run -- example.fl");
        process::exit(1);
    }

    let path = &args[1];

    if !path.ends_with(".fl") {
        eprintln!("Error: Flux files must have .fl extension");
        eprintln!("Example: cargo run -- example.fl");
        process::exit(1);
    }

    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            process::exit(1);
        }
    };

    // COMMENTED: Source code display (not Flux output)
    // println!("=== Loading: {} ===", path);
    // println!("{}", source);
    // println!("================{}", "=".repeat(path.len()));

    let tokens = match lexer::Lexer::new(&source).lex() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer Error: {:?}", e);
            process::exit(1);
        }
    };

    // COMMENTED: Token display (not Flux output)
    // println!("=== Tokens ===");
    // for (i, token) in tokens.iter().enumerate() {
    //     println!("{:3}: {:?}", i, token);
    // }
    // println!("==============");

    let program = match parser::Parser::new(tokens).parse() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Parser Error: {:?}", e);
            process::exit(1);
        }
    };

    // COMMENTED: AST display (not Flux output)
    // println!("=== AST ===");
    // for (i, stmt) in program.iter().enumerate() {
    //     println!("{:3}: {:?}", i, stmt);
    // }
    // println!("===========");

    let mut cg = codegen::Codegen::new();
    cg.compile(&program);

    // COMMENTED: IR display (not Flux output)
    // println!("=== Generated IR ===");
    // for (i, ir) in cg.code.iter().enumerate() {
    //     println!("{:3}: {:?}", i, ir);
    // }
    // println!("Functions: {:?}", cg.functions);
    // println!("===================");

    // COMMENTED: Execution header (not Flux output)
    // println!("=== Execution ===");
	println!(" ");
	println!(" ");

    let mut vm = vm::VM::new();
    vm.run(&cg.code, &cg.functions); // ONLY this produces actual Flux program output
    // COMMENTED: Execution footer (not Flux output)
    // println!("\n=================");
	println!(" ");
	println!(" ");

}