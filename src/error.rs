use crate::codegen::TargetShell;
use std::fmt;

#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub target: Option<TargetShell>,
    pub location: Option<String>,
}

impl CompileError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            target: None,
            location: None,
        }
    }

    pub fn with_target(mut self, target: TargetShell) -> Self {
        self.target = Some(target);
        self
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn unsupported_feature(feature: impl Into<String>, target: TargetShell) -> Self {
        Self {
            message: format!("{} is not supported in {:?} target", feature.into(), target),
            target: Some(target),
            location: None,
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "compile error: {}", self.message)?;
        if let Some(loc) = &self.location {
            write!(f, " at {}", loc)?;
        }
        Ok(())
    }
}

impl std::error::Error for CompileError {}
