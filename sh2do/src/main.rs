use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use tempfile::NamedTempFile;
use sh2c::driver::{self, CompileOptions};
use sh2c::codegen::TargetShell;

// Use internal library components (defined in lib.rs)
use sh2do::from_driver_code;
use sh2do::exit_code;

const HELP_TEXT: &str = "\
Usage: sh2do [flags] <file.sh2> [flags] -- [args...]
       sh2do [flags] '<snippet>' [flags] -- [args...]
       sh2do -

Compile and execute sh2 snippets or files.

Flags:
  -e, --emit     Emit compiled script (valid for file mode) and run it
  -o <path>      Explicit output path (implies run unless --no-exec)
  --emit-sh      Compile and emit shell to stdout, do not execute
  --no-exec      Alias of --emit-sh
  --target <t>   Target shell: bash (default) or posix
  --shell <s>    Override runtime shell (bash or sh)
  -h, --help     Show this help and exit
  -V, --version  Print version information and exit

Snippet input:
  <file.sh2>     File path to compile and run
  '<snippet>'    Inline sh2 code
  -              Read snippet from stdin

Arguments:
  Everything after -- is passed to the executed script

Exit codes:
  Compile error: exits with sh2c's code
  Runtime error: exits with script's code

Examples:
  sh2do script.sh2
  sh2do 'print(\"hi\")'
  sh2do 'run(\"ls\")'
";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}", err);
            // Default error exit
            ExitCode::from(1)
        }
    }
}

struct ParsedArgs {
    snippet_arg: String,
    target: Option<String>,
    shell: Option<String>,
    emit_and_run: bool, // --emit / -e
    out_path: Option<String>, // -o
    emit_stdout: bool, // --emit-sh / --no-exec
    passthrough: Vec<String>,
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = env::args().skip(1).collect();

    // Check for help/version early
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print!("{}", HELP_TEXT);
        return Ok(ExitCode::SUCCESS);
    }
    if args.iter().any(|arg| arg == "-V" || arg == "--version") {
        println!("sh2do {}", env!("CARGO_PKG_VERSION"));
        return Ok(ExitCode::SUCCESS);
    }

    // Split at --
    let (pre_args, passthrough) = match args.iter().position(|arg| arg == "--") {
        Some(pos) => (args[..pos].to_vec(), args[pos + 1..].to_vec()),
        None => (args, Vec::new()),
    };

    let mut parsed = ParsedArgs {
        snippet_arg: String::new(),
        target: None,
        shell: None,
        emit_and_run: false,
        out_path: None,
        emit_stdout: false,
        passthrough,
    };

    let mut snippet_found = false;
    let mut i = 0;
    while i < pre_args.len() {
        let arg = &pre_args[i];
        
        // Flags handling
        if arg == "--target" {
            if i + 1 < pre_args.len() {
                parsed.target = Some(pre_args[i + 1].clone());
                i += 2;
            } else {
                return Err("--target requires a value (bash|posix)".to_string());
            }
        } else if arg == "--shell" {
             if i + 1 < pre_args.len() {
                parsed.shell = Some(pre_args[i + 1].clone());
                i += 2;
            } else {
                return Err("--shell requires a value".to_string());
            }       
        } else if arg == "-o" {
            if i + 1 < pre_args.len() {
                parsed.out_path = Some(pre_args[i + 1].clone());
                i += 2;
            } else {
                return Err("-o requires a value".to_string());
            }
        } else if arg == "--emit" || arg == "-e" {
            parsed.emit_and_run = true;
            i += 1;
        } else if arg == "--emit-sh" || arg == "--no-exec" {
            parsed.emit_stdout = true;
            i += 1;
        } else if arg.starts_with("-") && arg != "-" {
            // Unknown flag
            return Err(format!("unexpected argument: {}", arg));
        } else {
            // Positional
            if !snippet_found {
                parsed.snippet_arg = arg.clone();
                snippet_found = true;
                i += 1;
            } else {
                return Err(format!("unexpected argument: {}", arg));
            }
        }
    }

    if !snippet_found {
        return Err("missing sh2 snippet or file argument (or '-')".to_string());
    }

    // Mode Detection
    // Rule: First non-flag ended with .sh2 => file-mode
    // Else if exists and is regular file => file-mode
    // Else if directory => Error
    // Else => inline
    let path_candidate = Path::new(&parsed.snippet_arg);
    let is_dir = path_candidate.is_dir();
    let is_file_ext = parsed.snippet_arg.ends_with(".sh2");
    let is_existing_file = path_candidate.exists() && path_candidate.is_file(); 
    
    // Explicit directory rejection
    if is_dir {
        return Err(format!("Path is a directory, expected file: {}", parsed.snippet_arg));
    }
    
    let is_file_mode = parsed.snippet_arg != "-" && (is_file_ext || is_existing_file);
    
    // Validation: --emit is file-mode only
    if parsed.emit_and_run && !is_file_mode {
        return Err("--emit is only valid when running a file; for inline, use --emit-sh > out.sh".to_string());
    }

    // Prepare Source
    let (src_path, _temp_src) = if is_file_mode {
        let p = Path::new(&parsed.snippet_arg);
        
        // Metadata check for existence (redundant but safe)
        if !p.exists() {
             return Err(format!("File not found: {}\nTry inline usage: sh2do 'run(\"echo\",\"hi\")'", parsed.snippet_arg));
        }
        
        // Readability check
        if let Err(e) = std::fs::File::open(p) {
            return Err(format!("Unable to read file: {} ({})\nTry inline usage: sh2do 'run(\"echo\",\"hi\")'", parsed.snippet_arg, e));
        }

        (p.to_path_buf(), None)
    } else {
        // Inline mode
        let content = read_snippet(parsed.snippet_arg.clone())?;
        let wrapped = wrap_snippet(&content);
        let t = NamedTempFile::new().map_err(|e| format!("failed to create temp file: {}", e))?;
        fs::write(t.path(), wrapped).map_err(|e| format!("failed to write snippet: {}", e))?;
        (t.path().to_path_buf(), Some(t))
    };
    
    // Check Target Shell mismatch
    // Default runtime: target=bash => bash, target=posix => sh
    let target_enum = match parsed.target.as_deref() {
        Some("posix") => TargetShell::Posix,
        Some("bash") | None => TargetShell::Bash,
        Some(other) => return Err(format!("Invalid target: {}", other)),
    };
    
    // Hardening: Validate --shell values
    if let Some(s) = &parsed.shell {
        if s != "bash" && s != "sh" {
             return Err(format!("Invalid shell: '{}'. Supported values: bash, sh", s));
        }
    }
    
    let runtime_shell_bin = parsed.shell.clone().unwrap_or_else(|| {
        match target_enum {
            TargetShell::Bash => "bash".to_string(),
            TargetShell::Posix => "sh".to_string(),
        }
    });

    // Validation: "bash target requires bash runtime; use --shell bash or --target posix."
    if matches!(target_enum, TargetShell::Bash) && runtime_shell_bin != "bash" {
         return Err("bash target requires bash runtime; use --shell bash or --target posix.".to_string());
    }

    // Determine Output Path
    // If emit_stdout: no output path needed for compilation (we read result from return)
    // If -o: use it
    // If --emit (and file mode): use <src_stem>.sh
    // Else: temp file
    
    let (out_path, _temp_out) = if parsed.emit_stdout {
        (None, None)
    } else if let Some(o) = parsed.out_path {
        (Some(PathBuf::from(o)), None)
    } else if parsed.emit_and_run && is_file_mode {
        // default SOURCE_BASENAME.sh next to source
        let stem = src_path.file_stem().ok_or("invalid filename")?;
        let mut p = src_path.parent().unwrap_or(Path::new(".")).to_path_buf();
        p.push(format!("{}.sh", stem.to_string_lossy()));
        (Some(p), None)
    } else {
        // temp file
        let t = NamedTempFile::new().map_err(|e| format!("failed to create output temp file: {}", e))?;
        (Some(t.path().to_path_buf()), Some(t))
    };
    
    // Compile
    let options = CompileOptions {
        target: target_enum,
        include_diagnostics: true,
        out_path: out_path.clone(),
        chmod_x: true, // sh2do is a runner, so we want +x
        ..Default::default()
    };
    
    // driver::compile_file returns Result<String, DriverError>
    let generated_code = match driver::compile_file(&src_path, options) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e.msg);
            return Ok(from_driver_code(e.code));
        }
    };

    if parsed.emit_stdout {
        print!("{}", generated_code);
        return Ok(ExitCode::SUCCESS);
    }
    
    // Execution
    let script_path = out_path.unwrap(); // Must exist if not emit_stdout
    
    let mut cmd = Command::new(&runtime_shell_bin);
    // bash -- <out_path> <args...>
    // Safety: Inject -- before script path to protect against script path starting with -
    cmd.arg("--");
    cmd.arg(&script_path);
    
    if !parsed.passthrough.is_empty() {
        cmd.args(&parsed.passthrough);
    }
    
    cmd.stdout(std::process::Stdio::inherit())
       .stderr(std::process::Stdio::inherit());
       
    let status = cmd.status()
        .map_err(|e| format!("failed to execute {}: {}", runtime_shell_bin, e))?;

    // Use robust status code mapping (defaults to 1 if None or out of range)
    Ok(exit_code::from_i32(status.code().unwrap_or(1)))
}
fn read_snippet(arg: String) -> Result<String, String> {
    if arg == "-" {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| e.to_string())?;
        Ok(buf)
    } else {
        Ok(arg)
    }
}

fn wrap_snippet(snippet: &str) -> String {
    let mut out = String::new();
    out.push_str("func main() {\n");
    for line in snippet.lines() {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("}\n");
    out
}
