mod lexer;
mod parser;
mod ast;
mod ir;
mod lower;
mod codegen;

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: sh2c <script.sh2>");
        std::process::exit(1);
    }
    let filename = &args[1];
    let src = fs::read_to_string(filename).unwrap();

    let tokens = lexer::lex(&src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);

    println!("{}", bash);
}
