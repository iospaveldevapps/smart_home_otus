use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum DeviceError {
    Io(io::Error),
    Protocol(String),
    NoData,
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceError::Io(error) => write!(f, "Ошибка ввода-вывода: {error}"),
            DeviceError::Protocol(message) => write!(f, "Ошибка протокола: {message}"),
            DeviceError::NoData => write!(f, "Данные от устройства ещё не получены"),
        }
    }
}

impl Error for DeviceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DeviceError::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for DeviceError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
