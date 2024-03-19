pub enum InternalError {
    Common(String),
    Multiple(Box<InternalError>, Box<InternalError>),
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
            InternalError::Multiple(err1, err2) => {
                format!("{}\n{}", err1.to_string(), err2.to_string())
            }
            InternalError::NetworkError(err) => format!("Network error: {}", err),
            InternalError::IOError(err) => format!("IO error: {}", err),
        }
    }
}

pub type InternalResult<T> = Result<T, InternalError>;
