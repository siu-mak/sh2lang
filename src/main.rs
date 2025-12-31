use sh2c::codegen;
use sh2c::codegen::TargetShell;
use sh2c::loader;
use sh2c::lower;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Suppress default panic output for string errors (our diagnostics),
    // but show it for everything else or if backtrace is requested.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // If RUST_BACKTRACE is set, use the default hook (prints thread panic info + backtrace)
        if std::env::var("RUST_BACKTRACE").is_ok() {
            default_hook(info);
        }
        // Otherwise, stay silent. catch_unwind in main() will handle printing the error message.
    }));

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut filename: Option<String> = None;
    let mut target = TargetShell::Bash;
    let mut include_diagnostics = true;
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
                eprintln!("--target requires an argument");
                process::exit(1);
            }
        } else if arg.starts_with("--target=") {
            let val = &arg["--target=".len()..];
            if val.is_empty() {
                eprintln!("--target requires an argument");
                process::exit(1);
            }
            target = parse_target(val);
            i += 1;
        } else if arg == "--no-diagnostics" {
            include_diagnostics = false;
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
            eprintln!("Unexpected argument: {}", arg);
            process::exit(1);
        } else {
            if filename.is_some() {
                eprintln!("Unexpected argument: {} (script already specified)", arg);
                process::exit(1);
            }
            filename = Some(arg.clone());
            i += 1;
        }
    }

    if (emit_ast as u8 + emit_ir as u8 + emit_sh as u8 + check as u8) > 1 {
        eprintln!("error: multiple action flags specified (choose only one of: --emit-ast, --emit-ir, --emit-sh, --check)");
        process::exit(1);
    }

    let filename = match filename {
        Some(f) => f,
        None => {
            print_usage();
            process::exit(1);
        }
    };

    // The _src variable is no longer used, as loader::load_program_with_imports
    // handles reading the file content internally.
    // let _src = match fs::read_to_string(&filename) {
    //     Ok(s) => s,
    //     Err(e) => {
    //         eprintln!("Failed to read {}: {}", filename, e);
    //         process::exit(1);
    //     }
    // };

    let result = std::panic::catch_unwind(|| -> Result<(), String> {
        // Loader handles reading, lexing, parsing, and resolving imports recursively
        let path = std::path::Path::new(&filename);
        
        // Base dir is the input file's parent directory.
        // This ensures paths are relative to the script location, which is more
        // robust than CWD if the compiler is run from elsewhere.
        let diag_base_dir = path.parent()
            .map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()));
            
        let mut ast = loader::load_program_with_imports(path)
            .map_err(|d| d.format(diag_base_dir.as_deref()))?;

        if emit_ast {
            ast.strip_spans();
            // Use debug formatting for deterministic snapshot structure
            println!("{:#?}", ast);
            return Ok(());
        }

        let ir = lower::lower_with_options(
            ast,
            &lower::LowerOptions {
                include_diagnostics,
                diag_base_dir: diag_base_dir.clone(), // Clone since Option is Copy but PathBuf isn't
            },
        );

        if emit_ir {
            // IR is typically Vec<Function>. Iterate and strip.
            // The `lower` function returns `Vec<sh2c::ir::Function>`.
            let mut ir_stripped = ir; // Take ownership to modify
            for f in &mut ir_stripped {
                 f.strip_spans();
            }
            println!("{:#?}", ir_stripped);
            return Ok(());
        }

        if check {
            // We lowered successfully. Now verify codegen doesn't panic/error.
            // For POSIX target, this also validates no bash-only constructs are emitted.
            codegen::emit_with_options_checked(
                &ir,
                codegen::CodegenOptions {
                    target,
                    include_diagnostics,
                },
            )?; // Propagate lint errors
            println!("OK");
            return Ok(());
        }

        // Default behavior or --emit-sh
        // Both print the generated shell script to stdout.
        let out = codegen::emit_with_options_checked(
            &ir,
            codegen::CodegenOptions {
                target,
                include_diagnostics,
            },
        )?; // Propagate lint errors
        
        print!("{}", out);
        Ok(())
    });

    match result {
        Ok(Ok(_)) => {
            // Success, output already printed or no output expected for check/emit-ast/emit-ir
        },
        Ok(Err(msg)) => {
            // It's a structured diagnostic message
             eprintln!("{}", msg);
             process::exit(2);
        }
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown compiler error".to_string()
            };
            
            // Format consistency: avoid double prefixes if the message already is a diagnostic
            if msg.starts_with("error: ") {
                eprintln!("{}", msg);
            } else {
                eprintln!("Error: {}", msg);
            }
            process::exit(2);
        }
    }
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
    eprintln!("  --no-diagnostics       Disable error location reporting and traps");
}
