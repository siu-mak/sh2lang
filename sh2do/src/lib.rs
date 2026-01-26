use std::process::ExitCode;

pub mod exit_code {
    use std::process::ExitCode;

    pub fn from_i32(code: i32) -> ExitCode {
        if code >= 0 && code <= 255 {
            ExitCode::from(code as u8)
        } else {
            ExitCode::from(1)
        }
    }
}

pub fn from_driver_code(code: i32) -> ExitCode {
    if code == 0 {
        // Driver generic error (0) -> Compile error (2)
        ExitCode::from(2)
    } else {
        exit_code::from_i32(code)
    }
}
