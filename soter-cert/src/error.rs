pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("STUN decoding error")]
    Stun(#[from] crate::stun::Error),
    #[error("unknown I/O error")]
    Io(#[from] std::io::Error),
    #[error("error generating certificate")]
    CertificateGeneration(#[from] rcgen::RcgenError),
    #[error("error generating server config")]
    TlsConfig(#[from] rustls::Error),
}
