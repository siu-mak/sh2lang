
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetShell {
    Bash,
    Posix,
}

impl std::fmt::Display for TargetShell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetShell::Bash => write!(f, "bash"),
            TargetShell::Posix => write!(f, "posix"),
        }
    }
}
