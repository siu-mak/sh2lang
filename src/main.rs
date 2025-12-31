use sh2c::codegen;
use sh2c::codegen::TargetShell;
use sh2c::loader;
use sh2c::lower;
use std::process;
use std::error::Error;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

struct Config {
    filename: String,
    target: TargetShell,
    include_diagnostics: bool,
    mode: Mode,
    out_path: Option<String>,
}

enum Mode {
    Default,
    Check,
    EmitAst,
    EmitIr,
    EmitSh,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
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

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--target" {
            if i + 1 < args.len() {
                target = parse_target(&args[i + 1]);
                i += 2;
            } else {
                eprintln!("error: --target requires an argument");
                process::exit(1);
            }
        } else if arg.starts_with("--target=") {
            let val = &arg["--target=".len()..];
            if val.is_empty() {
                eprintln!("error: --target requires an argument");
                process::exit(1);
            }
            target = parse_target(val);
            i += 1;
        } else if arg == "--no-diagnostics" {
            include_diagnostics = false;
            i += 1;
        } else if arg == "-o" || arg == "--out" {
            if i + 1 < args.len() {
                out_path = Some(args[i + 1].clone());
                i += 2;
            } else {
                eprintln!("error: {} requires an argument", arg);
                process::exit(1);
            }
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
            eprintln!("error: Unexpected argument: {}", arg);
            process::exit(1);
        } else {
            if filename.is_some() {
                eprintln!("error: Unexpected argument: {} (script already specified)", arg);
                process::exit(1);
            }
            filename = Some(arg.clone());
            i += 1;
        }
    }

    if check && out_path.is_some() {
        eprintln!("error: --check cannot be used with --out");
        print_usage();
        process::exit(2);
    }

    if (emit_ast as u8 + emit_ir as u8 + emit_sh as u8 + check as u8) > 1 {
        eprintln!("error: multiple action flags specified (choose only one of: --emit-ast, --emit-ir, --emit-sh, --check)");
        process::exit(1);
    }
    
    if emit_ast { mode = Mode::EmitAst; }
    else if emit_ir { mode = Mode::EmitIr; }
    else if emit_sh { mode = Mode::EmitSh; }
    else if check { mode = Mode::Check; }

    let filename = match filename {
        Some(f) => f,
        None => {
            print_usage();
            process::exit(1);
        }
    };

    let config = Config {
        filename,
        target,
        include_diagnostics,
        mode,
        out_path,
    };

    if let Err(e) = compile(config) {
        let msg = e.to_string();
        eprintln!("{}", msg);
        
        if msg.starts_with("Failed to write") {
             process::exit(1);
        } else {
             process::exit(2);
        }
    }
}

fn compile(config: Config) -> Result<(), Box<dyn Error>> {
    let path = std::path::Path::new(&config.filename);
    let diag_base_dir = path.parent()
        .map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()));
        
    let mut ast = loader::load_program_with_imports(path)
        .map_err(|d| d.format(diag_base_dir.as_deref()))?;

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
        )?; 
        println!("OK");
        return Ok(());
    }

    let out = codegen::emit_with_options_checked(
        &ir,
        codegen::CodegenOptions {
            target: config.target,
            include_diagnostics: config.include_diagnostics,
        },
    )?;
    
    if let Some(out_path) = config.out_path {
        std::fs::write(&out_path, out).map_err(|e| format!("Failed to write to {}: {}", out_path, e))?;
        
        #[cfg(unix)]
        {
            if let Ok(metadata) = std::fs::metadata(&out_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(perms.mode() | 0o111);
                let _ = std::fs::set_permissions(&out_path, perms);
            }
        }
    } else {
        print!("{}", out);
    }
    
    Ok(())
}

fn parse_target(s: &str) -> TargetShell {
    match s {
        "bash" => TargetShell::Bash,
        "posix" => TargetShell::Posix,
        _ => {
            eprintln!("Invalid target: {}. Supported: bash, posix", s);
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: sh2c [flags] <script.sh2> [flags]");
    eprintln!("Flags:");
    eprintln!("  --target <bash|posix>  Select output shell dialect (default: bash)");
    eprintln!("  -o, --out <file>       Write output to file instead of stdout (auto-chmod +x)");
    eprintln!("  --check                Check syntax and semantics without emitting code");
    eprintln!("  --no-diagnostics       Disable error location reporting and traps");
    eprintln!("  --emit-ast             Emit AST (debug)");
    eprintln!("  --emit-ir              Emit IR (debug)");
    eprintln!("  --emit-sh              Emit Shell (default)");
}
