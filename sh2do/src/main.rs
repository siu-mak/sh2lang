use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::{Command, ExitCode};

use tempfile::NamedTempFile;

fn find_sh2c() -> Result<std::path::PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("cannot determine current executable: {e}"))?;

    let dir = exe.parent()
        .ok_or("cannot determine executable directory")?;

    let sh2c = dir.join("sh2c");

    if sh2c.exists() {
        Ok(sh2c)
    } else {
        Err(format!(
            "sh2c not found next to sh2do (expected at {})",
            sh2c.display()
        ))
    }
}

const HELP_TEXT: &str = "\
Usage: sh2do '<snippet>'
       sh2do -
       sh2do '<snippet>' [flags] -- [args...]

Compile and execute sh2 snippets.

Flags:
  --emit-sh      Compile and emit shell to stdout, do not execute
  --no-exec      Alias of --emit-sh
  --target <t>   Target shell: bash (default) or posix
  -h, --help     Show this help and exit

Snippet input:
  '<snippet>'    sh2 code as argument
  -              Read snippet from stdin

Arguments:
  Everything after -- is passed to the executed script

Exit codes:
  Compile error: exits with sh2c's code
  Runtime error: exits with script's code

Examples:
  sh2do 'print(\"hi\")'
  sh2do 'run(\"ls\")'
";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = env::args().skip(1).collect();

    // Check for help flag early
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print!("{}", HELP_TEXT);
        return Ok(ExitCode::SUCCESS);
    }

    // Split arguments at '--' separator
    let separator_pos = args.iter().position(|arg| arg == "--");
    let (pre_args, passthrough_args) = match separator_pos {
        Some(pos) => {
            let pre = &args[..pos];
            let post = &args[pos + 1..];
            (pre.to_vec(), post.to_vec())
        }
        None => (args.clone(), Vec::new()),
    };

    // Parse flags and snippet from pre_args only
    let mut target: Option<String> = None;
    let mut emit_only = false;
    let mut snippet_arg: Option<String> = None;

    let mut i = 0;
    while i < pre_args.len() {
        let arg = &pre_args[i];
        
        if arg == "--target" {
            if i + 1 < pre_args.len() {
                target = Some(pre_args[i + 1].clone());
                i += 2;
            } else {
                return Err("--target requires a value (bash|posix)".to_string());
            }
        } else if arg == "--emit-sh" || arg == "--no-exec" {
            emit_only = true;
            i += 1;
        } else if snippet_arg.is_none() {
            snippet_arg = Some(arg.clone());
            i += 1;
        } else {
            return Err(format!("unexpected argument: {}", arg));
        }
    }

    let snippet_arg = snippet_arg.ok_or("missing sh2 snippet or '-'")?;
    let snippet = read_snippet(snippet_arg)?;

    let wrapped = wrap_snippet(&snippet);

    // Write snippet to temp .sh2
    let src = NamedTempFile::new().map_err(|e| e.to_string())?;
    fs::write(src.path(), wrapped).map_err(|e| e.to_string())?;

    // Output temp shell file
    let out = NamedTempFile::new().map_err(|e| e.to_string())?;

    // Invoke sh2c
    let sh2c_path = find_sh2c()?;
    let mut cmd = Command::new(sh2c_path);

    let target_shell = target.as_deref().unwrap_or("bash");

    if let Some(t) = &target {
        cmd.arg("--target").arg(t);
    }

    let status = cmd
        .arg(src.path())
        .arg("-o")
        .arg(out.path())
        .status()
        .map_err(|e| format!("failed to run sh2c: {e}"))?;

    if !status.success() {
        return Ok(ExitCode::from(status.code().unwrap_or(1) as u8));
    }

    // Branch on mode: emit or execute
    if emit_only {
        // Emit-only mode: print generated shell to stdout
        // Passthrough args are ignored in emit-only mode
        let shell = fs::read_to_string(out.path()).map_err(|e| e.to_string())?;
        print!("{}", shell);
        Ok(ExitCode::SUCCESS)
    } else {
        // Execute mode: run the generated shell script with passthrough args
        let interpreter = match target_shell {
            "bash" => "bash",
            "posix" => "sh",
            _ => "bash", // fallback to bash
        };

        let mut exec_cmd = Command::new(interpreter);
        exec_cmd
            .arg(out.path())
            .args(&passthrough_args)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());

        let exec_status = exec_cmd
            .status()
            .map_err(|e| format!("failed to execute {}: {}", interpreter, e))?;

        Ok(ExitCode::from(exec_status.code().unwrap_or(1) as u8))
    }
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
