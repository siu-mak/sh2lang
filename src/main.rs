mod lexer;
mod parser;
mod ast;
mod ir;
mod lower;
mod codegen;

use std::fs;
use codegen::TargetShell;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: sh2c <script.sh2> [--target {{bash|posix}}]");
        std::process::exit(1);
    }

    // Basic arg parsing (no clap)
    // sh2c <script> [--target <target>]
    // sh2c --target <target> <script> (harder to support without proper parsing, let's assume script is first or just iterate)
    
    let mut filename = String::new();
    let mut target = TargetShell::Bash;
    
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--target" {
            if i + 1 < args.len() {
                let t_str = &args[i+1];
                match t_str.as_str() {
                    "bash" => target = TargetShell::Bash,
                    "posix" => target = TargetShell::Posix,
                    _ => {
                        eprintln!("Invalid target: {}. Supported: bash, posix", t_str);
                        std::process::exit(1);
                    }
                }
                i += 2;
            } else {
                eprintln!("--target requires an argument");
                std::process::exit(1);
            }
        } else {
            if filename.is_empty() {
                filename = arg.clone();
                i += 1;
            } else {
                eprintln!("Unexpected argument: {}", arg);
                std::process::exit(1);
            }
        }
    }
    
    if filename.is_empty() {
        eprintln!("Usage: sh2c <script.sh2> [--target {{bash|posix}}]");
        std::process::exit(1);
    }
    
    let src = fs::read_to_string(&filename).unwrap();

    let tokens = lexer::lex(&src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit_with_target(&ir, target);

    println!("{}", bash);
}
