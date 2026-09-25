//! THEME errors: they tell a missing theme apart from an unreadable file or an unusable key.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Why the active theme could not be turned into a [`Theme`](super::Theme).
#[derive(Debug)]
pub enum ThemeError {
    /// No theme file exists at this path.
    NotFound(PathBuf),
    /// The theme file exists but could not be read.
    Unreadable(PathBuf, io::Error),
    /// The theme file is larger than the reader accepts.
    TooLarge(PathBuf, u64),
    /// The theme file is not valid TOML.
    InvalidToml(PathBuf, toml::de::Error),
    /// The theme omits a key the color contract requires.
    MissingKey(&'static str),
    /// `mode` holds something other than `dark` or `light`.
    InvalidMode(String),
    /// A color key holds something other than `#rrggbb`.
    InvalidColor(&'static str, String),
    /// The theme file could not be watched for changes.
    Watch(notify::Error),
    /// The change-notification fd could not be created.
    Signal(io::Error),
}

impl fmt::Display for ThemeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => {
                write!(formatter, "no theme found at {}", path.display())
            }
            Self::Unreadable(path, cause) => {
                write!(formatter, "cannot read {}: {cause}", path.display())
            }
            Self::TooLarge(path, size) => {
                write!(
                    formatter,
                    "the theme at {} is too large ({size} bytes)",
                    path.display()
                )
            }
            Self::InvalidToml(path, cause) => {
                write!(
                    formatter,
                    "invalid TOML in {}: {}",
                    path.display(),
                    cause.message()
                )
            }
            Self::MissingKey(key) => {
                write!(formatter, "the theme does not declare `{key}`")
            }
            Self::InvalidMode(value) => {
                write!(
                    formatter,
                    "`mode` expected \"dark\" or \"light\", got `{value}`"
                )
            }
            Self::InvalidColor(key, value) => {
                write!(
                    formatter,
                    "`{key}` expected an #rrggbb color, got `{value}`"
                )
            }
            Self::Watch(cause) => {
                write!(formatter, "cannot watch the theme for changes: {cause}")
            }
            Self::Signal(cause) => {
                write!(formatter, "cannot set up change notification: {cause}")
            }
        }
    }
}

impl Error for ThemeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Unreadable(_, cause) => Some(cause),
            Self::InvalidToml(_, cause) => Some(cause),
            Self::Watch(cause) => Some(cause),
            Self::Signal(cause) => Some(cause),
            _ => None,
        }
    }
}
