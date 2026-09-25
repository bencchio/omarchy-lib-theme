//! The mode a theme declares: it sets the direction surfaces and content are derived in.

use super::error::ThemeError;

/// Whether the theme is built on a dark or a light ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    pub fn parse(value: &str) -> Result<Self, ThemeError> {
        match value {
            "dark" => Ok(Self::Dark),
            "light" => Ok(Self::Light),
            other => Err(ThemeError::InvalidMode(other.to_owned())),
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }

    /// The other mode. Used when a transform makes a theme's colors read as the opposite ground,
    /// so mode-dependent derivations (surfaces, for one) keep reasoning in the right direction.
    pub fn flipped(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_both_modes() {
        assert_eq!(Mode::parse("dark").unwrap(), Mode::Dark);
        assert_eq!(Mode::parse("light").unwrap(), Mode::Light);
    }

    #[test]
    fn rejects_an_unknown_mode() {
        let error = Mode::parse("sepia").unwrap_err();
        assert!(matches!(error, ThemeError::InvalidMode(value) if value == "sepia"));
    }

    #[test]
    fn identifies_the_dark_mode() {
        assert!(Mode::Dark.is_dark());
        assert!(!Mode::Light.is_dark());
    }

    #[test]
    fn flips_to_the_other_mode_and_back() {
        assert_eq!(Mode::Dark.flipped(), Mode::Light);
        assert_eq!(Mode::Light.flipped(), Mode::Dark);
        assert_eq!(Mode::Dark.flipped().flipped(), Mode::Dark);
    }
}
