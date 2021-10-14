pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IncorrectCodeLength(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Error::IncorrectCodeLength(s) => {
                format!(
                    "code: \"{}\" is not of the correct length (found: {}, expected: {})",
                    s,
                    s.len(),
                    crate::cs::Code::LEN
                )
            }
        };

        write!(f, "{}", s)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
