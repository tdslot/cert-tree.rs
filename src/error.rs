use thiserror::Error;

#[derive(Error, Debug)]
pub enum CertError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("TLS error: {0}")]
    Tls(String),
    #[error("X.509 parsing error: {0}")]
    X509Parse(String),
    #[error("Invalid certificate format")]
    InvalidFormat,
    #[error("Certificate not found")]
    NotFound,
}

impl From<rustls::Error> for CertError {
    fn from(err: rustls::Error) -> Self {
        CertError::Tls(err.to_string())
    }
}
