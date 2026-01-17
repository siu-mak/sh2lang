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
    let mut args = env::args().skip(1);

    // Optional: --target <bash|posix>
    let mut target: Option<String> = None;

    let first = args.next().ok_or("missing sh2 snippet or '-'")?;

    let snippet = if first == "--target" {
        let t = args
            .next()
            .ok_or("--target requires a value (bash|posix)")?;
        target = Some(t.clone());

        let next = args.next().ok_or("missing sh2 snippet or '-'")?;
        read_snippet(next)?
    } else {
        read_snippet(first)?
    };

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

    // Execute generated shell script
    let interpreter = match target_shell {
        "bash" => "bash",
        "posix" => "sh",
        _ => "bash", // fallback to bash
    };

    let exec_status = Command::new(interpreter)
        .arg(out.path())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| format!("failed to execute {}: {}", interpreter, e))?;

    Ok(ExitCode::from(exec_status.code().unwrap_or(1) as u8))
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
