pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Bincode(lib::bincode::Error),
    Server(lib::cs::protocol::error::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<lib::bincode::Error> for Error {
    fn from(e: lib::bincode::Error) -> Self {
        Self::Bincode(e)
    }
}

impl From<lib::cs::protocol::error::Error> for Error {
    fn from(e: lib::cs::protocol::error::Error) -> Self {
        Self::Server(e)
    }
}
