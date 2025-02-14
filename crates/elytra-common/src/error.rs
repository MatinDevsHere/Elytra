use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ElytraError {
    IoError(std::io::Error),
    ProtocolError(String),
    ServerError(String),
}

impl fmt::Display for ElytraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElytraError::IoError(err) => write!(f, "IO error: {}", err),
            ElytraError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            ElytraError::ServerError(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

impl Error for ElytraError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ElytraError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ElytraError {
    fn from(err: std::io::Error) -> Self {
        ElytraError::IoError(err)
    }
} 