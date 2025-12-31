use sh2c::codegen;
use sh2c::codegen::TargetShell;
use sh2c::loader;
use sh2c::lower;
use std::process;
use std::fmt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

struct Config {
    filename: String,
    target: TargetShell,
    include_diagnostics: bool,
    mode: Mode,
    out_path: Option<String>,
    chmod_x: bool,
}

enum Mode {
    Default,
    Check,
    EmitAst,
    EmitIr,
    EmitSh,
}

struct CliError {
    code: i32,
    msg: String,
    show_usage: bool,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl CliError {
    fn compile(msg: impl Into<String>) -> Self {
        Self { code: 2, msg: msg.into(), show_usage: false }
    }
    
    fn io(msg: impl Into<String>) -> Self {
        Self { code: 1, msg: msg.into(), show_usage: false }
    }

    fn usage(msg: impl Into<String>) -> Self {
        Self { code: 1, msg: msg.into(), show_usage: true }
    }

    fn usage_with_code(msg: impl Into<String>, code: i32) -> Self {
        Self { code, msg: msg.into(), show_usage: true }
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
     \x20 -h, --help             Print help information"
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
    let mut target = TargetShell::Bash;
    let mut include_diagnostics = true;
    let mut mode = Mode::Default;
    let mut out_path: Option<String> = None;
    
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
        } else if arg == "--target" {
            if i + 1 < args.len() {
                target = parse_target(&args[i + 1])?;
                i += 2;
            } else {
                return Err(CliError::usage("error: --target requires an argument"));
            }
        } else if arg.starts_with("--target=") {
            let val = &arg["--target=".len()..];
            if val.is_empty() {
                return Err(CliError::usage("error: --target requires an argument"));
            }
            target = parse_target(val)?;
            i += 1;
        } else if arg == "--no-diagnostics" {
            include_diagnostics = false;
            i += 1;
        } else if arg == "-o" || arg == "--out" {
            if i + 1 < args.len() {
                out_path = Some(args[i + 1].clone());
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

    if check && out_path.is_some() {
        return Err(CliError::usage_with_code("error: --check cannot be used with --out", 2));
    }
    
    if chmod_x_flag.is_some() && out_path.is_none() {
        return Err(CliError::usage("error: --no-chmod-x/--chmod-x require --out"));
    }

    if (emit_ast as u8 + emit_ir as u8 + emit_sh as u8 + check as u8) > 1 {
         return Err(CliError::usage("error: multiple action flags specified (choose only one of: --emit-ast, --emit-ir, --emit-sh, --check)"));
    }
    
    if emit_ast { mode = Mode::EmitAst; }
    else if emit_ir { mode = Mode::EmitIr; }
    else if emit_sh { mode = Mode::EmitSh; }
    else if check { mode = Mode::Check; }

    let filename = match filename {
        Some(f) => f,
        None => {
             return Err(CliError::usage("error: missing input file"));
        }
    };

    Ok(Config {
        filename,
        target,
        include_diagnostics,
        mode,
        out_path,
        chmod_x: chmod_x_flag.unwrap_or(true),
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
    let diag_base_dir = path.parent()
        .map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()));
        
    let mut ast = loader::load_program_with_imports(path)
        .map_err(|d| CliError::compile(d.format(diag_base_dir.as_deref())))?;

    if let Mode::EmitAst = config.mode {
        ast.strip_spans();
        println!("{:#?}", ast);
        return Ok(());
    }

    let ir = lower::lower_with_options(
        ast,
        &lower::LowerOptions {
            include_diagnostics: config.include_diagnostics,
            diag_base_dir: diag_base_dir.clone(),
        },
    );

    if let Mode::EmitIr = config.mode {
        let mut ir_stripped = ir;
        for f in &mut ir_stripped {
             f.strip_spans();
        }
        println!("{:#?}", ir_stripped);
        return Ok(());
    }

    if let Mode::Check = config.mode {
        codegen::emit_with_options_checked(
            &ir,
            codegen::CodegenOptions {
                target: config.target,
                include_diagnostics: config.include_diagnostics,
            },
        ).map_err(|e| CliError::compile(e.to_string()))?;
        println!("OK");
        return Ok(());
    }

    let out = codegen::emit_with_options_checked(
        &ir,
        codegen::CodegenOptions {
            target: config.target,
            include_diagnostics: config.include_diagnostics,
        },
    ).map_err(|e| CliError::compile(e.to_string()))?;
    
    if let Some(out_path) = config.out_path {
        std::fs::write(&out_path, out).map_err(|e| CliError::io(format!("Failed to write to {}: {}", out_path, e)))?;
        
        #[cfg(unix)]
        {
            if config.chmod_x {
                if let Ok(metadata) = std::fs::metadata(&out_path) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(perms.mode() | 0o111);
                    let _ = std::fs::set_permissions(&out_path, perms);
                }
            }
        }
    } else {
        print!("{}", out);
    }
    
    Ok(())
}
