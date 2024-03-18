pub enum InternalError {
    Common(String),
    NetworkError(String),
    IOError(String),
}

impl From<reqwest::Error> for InternalError {
    fn from(err: reqwest::Error) -> Self {
        InternalError::NetworkError(err.to_string())
    }
}

impl From<std::io::Error> for InternalError {
    fn from(err: std::io::Error) -> Self {
        InternalError::IOError(err.to_string())
    }
}

impl ToString for InternalError {
    fn to_string(&self) -> String {
        match self {
            InternalError::Common(err) => err.clone(),
            InternalError::NetworkError(err) => format!("Network error: {}", err),
            InternalError::IOError(err) => format!("IO error: {}", err),
        }
    }
}

pub type InternalResult<T> = Result<T, InternalError>;
