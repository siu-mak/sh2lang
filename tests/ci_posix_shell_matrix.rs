mod common;
use common::*;
use sh2c::codegen::TargetShell;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ENV_LOCK: Mutex<()> = Mutex::new(());
}

#[test]
fn test_strict_mode_enforcement_missing_shell() {
    let _guard = ENV_LOCK.lock().unwrap();
    
    unsafe {
        std::env::set_var("SH2C_POSIX_SHELL", "shell_that_does_not_exist_12345");
    }
    
    let result = std::panic::catch_unwind(|| {
        assert_exec_matches_fixture_target("truthy_empty_string", TargetShell::Posix);
    });
    
    unsafe {
        std::env::remove_var("SH2C_POSIX_SHELL");
    }

    assert!(result.is_err(), "Expected panic when SH2C_POSIX_SHELL points to missing shell");
}

#[test]
fn test_strict_mode_enforcement_valid_shell() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("SH2C_POSIX_SHELL", "sh");
    }
    
    let result = std::panic::catch_unwind(|| {
        assert_exec_matches_fixture_target("truthy_empty_string", TargetShell::Posix);
    });
    
    unsafe {
        std::env::remove_var("SH2C_POSIX_SHELL");
    }

    assert!(result.is_ok(), "Expected success when SH2C_POSIX_SHELL points to valid 'sh'");
}
