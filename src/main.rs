use sh2c::codegen;
use sh2c::codegen::TargetShell;
use sh2c::loader;
use sh2c::lower;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Suppress default panic output for string errors (our diagnostics),
    // but show it for everything else or if backtrace is requested.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            Some(*s)
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            Some(s.as_str())
        } else {
            None
        };

        if let Some(msg) = msg {
            if msg.starts_with("error: ") && std::env::var("RUST_BACKTRACE").is_err() {
                // It's a structured diagnostic, let catch_unwind handle printing it cleanly.
                return;
            }
        }
        
        // Otherwise, it's an unexpected panic or backtrace is requested.
        default_hook(info);
    }));

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut filename: Option<String> = None;
    let mut target = TargetShell::Bash;
    let mut include_diagnostics = true;

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

    let filename = match filename {
        Some(f) => f,
        None => {
            print_usage();
            process::exit(1);
        }
    };

    let _src = match fs::read_to_string(&filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {}: {}", filename, e);
            process::exit(1);
        }
    };

    let result = std::panic::catch_unwind(|| {
        // Loader handles reading, lexing, parsing, and resolving imports recursively
        let path = std::path::Path::new(&filename);
        let ast = loader::load_program_with_imports(path);
        
        // Base dir is the input file's parent directory.
        // This ensures paths are relative to the script location, which is more
        // robust than CWD if the compiler is run from elsewhere.
        let diag_base_dir = path.parent()
            .map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()));

        let ir = lower::lower_with_options(
            ast,
            &lower::LowerOptions {
                include_diagnostics,
                diag_base_dir,
            },
        );
        codegen::emit_with_options(
            &ir,
            codegen::CodegenOptions {
                target,
                include_diagnostics,
            },
        )
    });

    match result {
        Ok(out) => print!("{}", out),
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
