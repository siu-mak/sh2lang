use sh2c::driver::{self, CompileOptions, DriverError, Mode};
use sh2c::codegen::TargetShell;
use std::process;

struct Config {
    filename: String,
    options: CompileOptions,
}

struct CliError {
    code: i32,
    msg: String,
    show_usage: bool,
}

impl CliError {
    fn usage(msg: impl Into<String>) -> Self {
        Self { code: 1, msg: msg.into(), show_usage: true }
    }
    
    fn usage_with_code(msg: impl Into<String>, code: i32) -> Self {
        Self { code, msg: msg.into(), show_usage: true }
    }

    fn from_driver(err: DriverError) -> Self {
        Self { code: err.code, msg: err.msg, show_usage: false }
    }
}

fn usage_text() -> &'static str {
    "Usage: sh2c [flags] <script.sh2> [flags]\n\
     Flags:\n\
     \x20 --target <bash|posix>  Select output shell dialect (default: bash)\n\
     \x20 -o, --out <file>       Write output to file instead of stdout (auto-chmod +x)\n\
     \x20 --check                Check syntax and semantics without emitting code\n\
     \x20 --no-diagnostics       Disable error location reporting and traps\n\
     \x20 --no-chmod-x           Do not set executable bit on output file\n\
     \x20 --chmod-x              Set executable bit on output file (default)\n\
     \x20 --emit-ast             Emit AST (debug)\n\
     \x20 --emit-ir              Emit IR (debug)\n\
     \x20 --emit-sh              Emit Shell (default)\n\
     \x20 -h, --help             Print help information\n\
     \x20 -V, --version          Print version information and exit"
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let config = match parse_args(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e.msg);
            if e.show_usage {
                eprintln!("{}", usage_text());
            }
            process::exit(e.code);
        }
    };

    if let Err(e) = compile(config) {
        eprintln!("{}", e.msg);
        if e.show_usage {
            eprintln!("{}", usage_text());
        }
        process::exit(e.code);
    }
}

fn parse_args(args: Vec<String>) -> Result<Config, CliError> {
    if args.len() < 2 {
        return Err(CliError::usage("error: missing input file"));
    }

    let mut filename: Option<String> = None;
    let mut options = CompileOptions::default();
    // Default CLI behavior: chmod_x=true is documented default in usage text.
    // But library default is false. We should set it to true here for CLI parity.
    options.chmod_x = true;
    
    let mut emit_ast = false;
    let mut emit_ir = false;
    let mut emit_sh = false;
    let mut check = false;
    
    let mut chmod_x_flag: Option<bool> = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            println!("{}", usage_text());
            process::exit(0);
        } else if arg == "-V" || arg == "--version" {
            println!("sh2c {}", env!("CARGO_PKG_VERSION"));
            process::exit(0);
        } else if arg == "--target" {
            if i + 1 < args.len() {
                options.target = parse_target(&args[i + 1])?;
                i += 2;
            } else {
                return Err(CliError::usage("error: --target requires an argument"));
            }
        } else if arg.starts_with("--target=") {
            let val = &arg["--target=".len()..];
            if val.is_empty() {
                return Err(CliError::usage("error: --target requires an argument"));
            }
            options.target = parse_target(val)?;
            i += 1;
        } else if arg == "--no-diagnostics" {
            options.include_diagnostics = false;
            i += 1;
        } else if arg == "-o" || arg == "--out" {
            if i + 1 < args.len() {
                options.out_path = Some(std::path::PathBuf::from(&args[i + 1]));
                i += 2;
            } else {
                return Err(CliError::usage(format!("error: {} requires an argument", arg)));
            }
        } else if arg == "--no-chmod-x" {
             if let Some(true) = chmod_x_flag {
                 return Err(CliError::usage("error: --no-chmod-x cannot be used with --chmod-x"));
             }
             chmod_x_flag = Some(false);
             i += 1;
        } else if arg == "--chmod-x" {
             if let Some(false) = chmod_x_flag {
                 return Err(CliError::usage("error: --no-chmod-x cannot be used with --chmod-x"));
             }
             chmod_x_flag = Some(true);
             i += 1;
        } else if arg == "--emit-ast" {
            emit_ast = true;
            i += 1;
        } else if arg == "--emit-ir" {
            emit_ir = true;
            i += 1;
        } else if arg == "--emit-sh" {
            emit_sh = true;
            i += 1;
        } else if arg == "--check" {
            check = true;
            i += 1;
        } else if arg.starts_with("-") {
             return Err(CliError::usage(format!("error: Unexpected argument: {}", arg)));
        } else {
            if filename.is_some() {
                 return Err(CliError::usage(format!("error: Unexpected argument: {} (script already specified)", arg)));
            }
            filename = Some(arg.clone());
            i += 1;
        }
    }

    if check && options.out_path.is_some() {
        return Err(CliError::usage_with_code("error: --check cannot be used with --out", 2));
    }
    
    if chmod_x_flag.is_some() && options.out_path.is_none() {
        return Err(CliError::usage("error: --no-chmod-x/--chmod-x require --out"));
    }

    if (emit_ast as u8 + emit_ir as u8 + emit_sh as u8 + check as u8) > 1 {
         return Err(CliError::usage("error: multiple action flags specified (choose only one of: --emit-ast, --emit-ir, --emit-sh, --check)"));
    }
    
    if emit_ast { options.mode = Mode::EmitAst; }
    else if emit_ir { options.mode = Mode::EmitIr; }
    else if emit_sh { options.mode = Mode::EmitSh; }
    else if check { options.mode = Mode::Check; }

    if let Some(flag) = chmod_x_flag {
        options.chmod_x = flag;
    }

    let filename = match filename {
        Some(f) => f,
        None => {
             return Err(CliError::usage("error: missing input file"));
        }
    };

    Ok(Config {
        filename,
        options,
    })
}

fn parse_target(s: &str) -> Result<TargetShell, CliError> {
    match s {
        "bash" => Ok(TargetShell::Bash),
        "posix" => Ok(TargetShell::Posix),
        _ => Err(CliError::usage(format!("Invalid target: {}. Supported: bash, posix", s))),
    }
}

fn compile(config: Config) -> Result<(), CliError> {
    let path = std::path::Path::new(&config.filename);
    
    let mode = config.options.mode;
    let has_out_path = config.options.out_path.is_some();
    
    let result = driver::compile_file(path, config.options)
        .map_err(CliError::from_driver)?;
        
    // Driver handles writing to file if out_path is set.
    // If not, it returns the content (or "OK" for check). 
    // We should print it unless out_path was set (but compile_file returns string anyway).
    // CLI logic:
    // If Mode::Check: prints "OK" (Driver returns "OK").
    // If Mode::EmitAst/Ir: Driver returns debug string.
    // If Mode::EmitSh: Driver returns shell code.
    // Driver writes to file if out_path is set.
    // So checking has_out_path here is correct.
    
    if !has_out_path {
        match mode {
             Mode::Default | Mode::EmitSh => print!("{}", result),
             Mode::Check | Mode::EmitAst | Mode::EmitIr => println!("{}", result),
        }
    } else if mode == Mode::Check {
        // Edge case: check with out_path? CLI parser rejects check+out.
        // So this branch is unreachable or fine.
        println!("{}", result); 
    }
    
    Ok(())
}
