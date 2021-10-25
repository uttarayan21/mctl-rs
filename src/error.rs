use derive_more::Display;
// use xdg::BaseDirectoriesError;

#[derive(Debug, Display)]
#[display(fmt = "{}", kind)]
pub struct Error {
    pub kind: ErrorKind,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, source: None }
    }
}

impl From<mpd::error::Error> for Error {
    fn from(error: mpd::error::Error) -> Self {
        Self::new(ErrorKind::MPDError(error))
    }
}
impl From<mpris::DBusError> for Error {
    fn from(error: mpris::DBusError) -> Self {
        Self::new(ErrorKind::DBusError(error))
    }
}
impl From<mpris::FindingError> for Error {
    fn from(error: mpris::FindingError) -> Self {
        Self::new(ErrorKind::MPRISError(error))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Self::new(ErrorKind::ParsingError(error))
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::new(ErrorKind::IOError(error))
    }
}

impl From<xdg::BaseDirectoriesError> for Error {
    fn from(error: xdg::BaseDirectoriesError) -> Self {
        Self::new(ErrorKind::BaseDirectoriesError(error))
    }
}

#[derive(Debug, Display)]
pub enum ErrorKind {
    ParsingError(serde_yaml::Error),
    IOError(std::io::Error),
    MPDError(mpd::error::Error),
    DBusError(mpris::DBusError),
    MPRISError(mpris::FindingError),
    BaseDirectoriesError(xdg::BaseDirectoriesError),
    UnknownOperation,
}
