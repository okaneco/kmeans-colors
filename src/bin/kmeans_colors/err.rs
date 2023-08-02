#[derive(Debug)]
pub enum CliError {
    File(std::io::Error),
    Parse(std::num::ParseIntError),
    Time(std::time::SystemTimeError),
    InvalidHex,
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::File(err)
    }
}

impl From<std::num::ParseIntError> for CliError {
    fn from(err: std::num::ParseIntError) -> CliError {
        CliError::Parse(err)
    }
}

impl From<std::time::SystemTimeError> for CliError {
    fn from(err: std::time::SystemTimeError) -> CliError {
        CliError::Time(err)
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::File(err) => write!(f, "{err}"),
            CliError::Parse(err) => write!(f, "{err}"),
            CliError::Time(err) => write!(f, "{err}"),
            CliError::InvalidHex => write!(f, "Invalid hex color, must be 3 or 6 digts"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::File(err) => Some(err),
            CliError::Parse(err) => Some(err),
            CliError::Time(err) => Some(err),
            CliError::InvalidHex => None,
        }
    }
}
