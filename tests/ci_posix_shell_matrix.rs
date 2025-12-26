mod common;
use common::*;
use sh2c::codegen::TargetShell;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ENV_LOCK: Mutex<()> = Mutex::new(());
}

// RAII guard to safe-guard env var state
struct EnvGuard {
    key: String,
    old_val: Option<String>,
}

impl EnvGuard {
    fn set(key: &str, val: &str) -> Self {
        let old_val = std::env::var(key).ok();
        unsafe {
            std::env::set_var(key, val);
        }
        EnvGuard {
            key: key.to_string(),
            old_val,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(v) = &self.old_val {
            unsafe {
                std::env::set_var(&self.key, v);
            }
        } else {
            unsafe {
                std::env::remove_var(&self.key);
            }
        }
    }
}

#[test]
fn ci_strict_posix_rejects_unknown_shell() {
    let _lock = ENV_LOCK.lock().unwrap();
    
    // Set invalid shell -> expect panic with specific message
    let _env = EnvGuard::set("SH2C_POSIX_SHELL", "shell_that_does_not_exist_12345");
    
    let result = std::panic::catch_unwind(|| {
        assert_exec_matches_fixture_target("truthy_empty_string", TargetShell::Posix);
    });
    
    match result {
        Ok(_) => panic!("Expected panic but found success"),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                *s
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.as_str()
            } else {
                "Unknown panic message"
            };
            assert!(msg.contains("Invalid SH2C_POSIX_SHELL"), "Got unexpected panic message: {}", msg);
        }
    }
}

#[test]
fn ci_strict_posix_runs_under_dash() {
    let _lock = ENV_LOCK.lock().unwrap();

    // Set valid shell "dash". 
    // This requires "dash" to be installed on the system running this test.
    // If dash is missing, this test fails (which is valid for strict CI environments).
    let _env = EnvGuard::set("SH2C_POSIX_SHELL", "dash");
    
    let result = std::panic::catch_unwind(|| {
        assert_exec_matches_fixture_target("truthy_empty_string", TargetShell::Posix);
    });
    
    if let Err(e) = result {
         let msg = if let Some(s) = e.downcast_ref::<&str>() {
                *s
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.as_str()
            } else {
                "Unknown panic message"
            };
        // If dash isn't installed, we might see the panic from our harness.
        // But for CI, we want this to succeed. For local dev without dash, this fails.
        // User requirements say: "This test MUST fail on systems without dash".
        panic!("Strict POSIX test failed (dash missing?): {}", msg);
    }
}
