use std::process::ExitCode;
use sh2do::exit_code::from_i32;
use sh2do::from_driver_code;

#[test]
fn exit_code_mapping_negative_defaults_to_1() {
    assert_eq!(from_i32(-1), ExitCode::from(1));
}

#[test]
fn exit_code_mapping_256_defaults_to_1() {
    assert_eq!(from_i32(256), ExitCode::from(1));
}

#[test]
fn exit_code_mapping_9999_defaults_to_1() {
    assert_eq!(from_i32(9999), ExitCode::from(1));
}

#[test]
fn exit_code_mapping_in_range_preserved() {
    assert_eq!(from_i32(0), ExitCode::from(0));
    assert_eq!(from_i32(2), ExitCode::from(2));
    assert_eq!(from_i32(255), ExitCode::from(255));
}

#[test]
fn driver_code_zero_is_mapped_to_compile_error_2() {
    assert_eq!(from_driver_code(0), ExitCode::from(2));
    assert_eq!(from_driver_code(2), ExitCode::from(2));
    assert_eq!(from_driver_code(256), ExitCode::from(1));
}
