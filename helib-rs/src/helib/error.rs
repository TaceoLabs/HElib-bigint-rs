use super::CLong;
use thiserror::Error;

/// An Error enum capturing the errors produced by this crate.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// Invalid pointer provided
    #[error("Invalid pointer")]
    Pointer,
    /// Too few slots available
    #[error("Too few slots available")]
    BatchSlots,
    /// Some other error has occured.
    #[error("Err: {0}")]
    Other(String),
}

impl From<String> for Error {
    fn from(mes: String) -> Self {
        Self::Other(mes)
    }
}
impl From<&str> for Error {
    fn from(mes: &str) -> Self {
        Self::Other(mes.to_owned())
    }
}

impl Error {
    pub(crate) fn error_from_return(ret: CLong) -> Result<(), Error> {
        match ret {
            0 => Ok(()),
            0x80004003 => Err(Error::Pointer),
            _ => Err(Error::Other("Error: Unknown Error".to_string())),
        }
    }
}
