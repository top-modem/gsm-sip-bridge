use std::fmt;

#[derive(Debug)]
pub enum PjsipError {
    InitFailed(String),
}

impl fmt::Display for PjsipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PjsipError::InitFailed(msg) => write!(f, "PJSIP init failed: {msg}"),
        }
    }
}

impl std::error::Error for PjsipError {}
