use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use assert_cmd::Command as AssertCommand;

/// Conformance test context for a single fixture
struct ConformanceTest {
    name: String,
    fixture_path: PathBuf,
}

impl ConformanceTest {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fixture_path: PathBuf::from(format!("tests/fixtures/stdlib/{}.sh2", name)),
        }
    }
    
    /// Compile the fixture with the given target
    fn compile(&self, target: &str) -> Result<String, String> {
        let mut cmd = AssertCommand::cargo_bin("sh2c").unwrap();
        
        cmd.arg("--target")
            .arg(target)
            .arg("--no-diagnostics") // Cleaner output for conformance
            .arg(&self.fixture_path);
        
        let output = cmd.output().expect("Failed to run sh2c");
        
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Execute a shell script and capture result
    fn execute(&self, shell: &str, script: &str) -> ExecutionResult {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("script.sh");
        
        fs::write(&script_path, script).expect("Failed to write script");
        
        let output = Command::new(shell)
            .arg(&script_path)
            .current_dir(temp_dir.path())
            .output()
            .expect(&format!("Failed to execute {}", shell));
        
        ExecutionResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status.code().unwrap_or(-1),
        }
    }
    
    /// Run full conformance test for this fixture
    fn run(&self) {
        let update_snapshots = std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok();
        
        // Test bash target
        self.test_target("bash", "bash", update_snapshots);
        
        // Test posix target under bash
        self.test_target("posix", "bash", update_snapshots);
        
        // Test posix target under dash (if available)
        if Command::new("dash").arg("--version").output().is_ok() {
            self.test_target_dash("posix", "dash", update_snapshots);
        } else {
            eprintln!("Warning: dash not available, skipping dash conformance tests");
        }
    }
    
    fn test_target(&self, target: &str, shell: &str, update_snapshots: bool) {
        let script = self.compile(target)
            .expect(&format!("Failed to compile {} with target {}", self.name, target));
        
        let result = self.execute(shell, &script);
        
        let variant = if target == "bash" {
            "bash".to_string()
        } else {
            format!("posix-{}", shell)
        };
        
        self.verify_result(&variant, &result, update_snapshots);
    }
    
    fn test_target_dash(&self, target: &str, shell: &str, update_snapshots: bool) {
        let script = self.compile(target)
            .expect(&format!("Failed to compile {} with target {}", self.name, target));
        
        let result = self.execute(shell, &script);
        
        let variant = format!("posix-{}", shell);
        self.verify_result(&variant, &result, update_snapshots);
    }
    
    fn verify_result(&self, variant: &str, result: &ExecutionResult, update_snapshots: bool) {
        let base_path = format!("tests/fixtures/stdlib/{}.{}", self.name, variant);
        
        // Stdout
        let stdout_path = format!("{}.stdout.expected", base_path);
        self.verify_snapshot(&stdout_path, &result.stdout, update_snapshots, "stdout");
        
        // Stderr
        let stderr_path = format!("{}.stderr.expected", base_path);
        self.verify_snapshot(&stderr_path, &result.stderr, update_snapshots, "stderr");
        
        // Status
        let status_path = format!("{}.status.expected", base_path);
        let status_str = format!("{}\n", result.status);
        self.verify_snapshot(&status_path, &status_str, update_snapshots, "status");
    }
    
    fn verify_snapshot(&self, path: &str, actual: &str, update: bool, desc: &str) {
        if update {
            fs::write(path, actual).expect(&format!("Failed to write {}", path));
        } else {
            let expected = fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("Missing snapshot file: {}", path));
            
            assert_eq!(
                actual, expected,
                "{} {} mismatch for {}",
                self.name, desc, path
            );
        }
    }
}

struct ExecutionResult {
    stdout: String,
    stderr: String,
    status: i32,
}

#[test]
fn test_string_ops() {
    ConformanceTest::new("string_ops").run();
}

#[test]
fn test_bool_truthiness() {
    ConformanceTest::new("bool_truthiness").run();
}

#[test]
fn test_capture_and_status() {
    ConformanceTest::new("capture_and_status").run();
}
