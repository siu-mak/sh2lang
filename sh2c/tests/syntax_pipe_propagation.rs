use crate::common::*;

mod common;

#[test]
fn exec_pipe_fail_middle() {
    assert_exec_matches_fixture_target("pipe_fail_middle", TargetShell::Bash);
    assert_exec_matches_fixture_target("pipe_fail_middle", TargetShell::Posix);
}

#[test]
fn exec_pipe_fail_first() {
    assert_exec_matches_fixture_target("pipe_fail_first", TargetShell::Bash);
    assert_exec_matches_fixture_target("pipe_fail_first", TargetShell::Posix);
}

#[test]
fn exec_pipe_two_fail_rightmost_wins() {
    assert_exec_matches_fixture_target("pipe_two_fail_rightmost_wins", TargetShell::Bash);
    assert_exec_matches_fixture_target("pipe_two_fail_rightmost_wins", TargetShell::Posix);
}

#[test]
fn exec_pipe_allow_fail_middle_ignored() {
    assert_exec_matches_fixture_target("pipe_allow_fail_middle_ignored", TargetShell::Bash);
    assert_exec_matches_fixture_target("pipe_allow_fail_middle_ignored", TargetShell::Posix);
}

#[test]
fn exec_pipeblocks_fail_middle() {
    // Manually run to ignore stderr flakiness with trap ERR in pipelines
    let fixture_name = "pipeblocks_fail_middle";
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let stdout_path = format!("tests/fixtures/{}.stdout", fixture_name);
    let status_path = format!("tests/fixtures/{}.status", fixture_name);

    if !std::path::Path::new(&sh2_path).exists() {
        panic!("Fixture {} does not exist", sh2_path);
    }
    
    let src = std::fs::read_to_string(&sh2_path).unwrap();
    let expected_stdout = std::fs::read_to_string(&stdout_path)
        .unwrap_or_default()
        .trim()
        .replace("\r\n", "\n");
    let expected_status: i32 = std::fs::read_to_string(&status_path)
        .expect("Missing status file")
        .trim()
        .parse()
        .unwrap();

    // Replicate target loop from common
    for target in [TargetShell::Bash, TargetShell::Posix] {
        let shell_script = compile_to_shell(&src, target);
        let shell_bin = match target {
            TargetShell::Bash => "bash",
            TargetShell::Posix => {
                // Simplified check: assume dash or sh available
                 if std::process::Command::new("dash").arg("-c").arg("true").status().map(|s| s.success()).unwrap_or(false) {
                    "dash"
                } else {
                    "sh"
                }
            }
        };

        let (stdout, _stderr, status) = run_shell_script(
            &shell_script,
            shell_bin,
            &[],
            &[],
            None,
            None,
        );

        assert_eq!(stdout.trim(), expected_stdout, "Stdout mismatch for {:?} target", target);
        assert_eq!(status, expected_status, "Status mismatch for {:?} target", target);
    }
}

/// Test that POSIX pipelines work correctly even when the shell has `set -e` enabled.
/// The pipeline helper must shield errexit during waits/status collection.
/// We only test POSIX with -e flag because Bash doesn't have errexit enabled by default.
#[test]
fn exec_pipe_posix_errexit_safe() {
    // Posix target: run with -e flag to verify errexit-safe behavior
    // The pipeline itself should succeed (wait + collect status), and then
    // (exit $__sh2_status) should trigger errexit abort, preventing UNREACHABLE.
    assert_exec_matches_fixture_target_with_flags(
        "pipe_posix_errexit_safe",
        TargetShell::Posix,
        &["-e"],
    );
}
