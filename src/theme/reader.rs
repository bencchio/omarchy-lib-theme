//! Reading the active theme: from a path on disk to a `Theme`, or to an error that says why not.

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use toml::Table;

use super::Theme;
use super::colors::ThemeColors;
use super::error::ThemeError;
use super::mode::Mode;

const THEME_FILE: &str = ".config/omarchy/current/theme/colors.toml";

/// A theme is a handful of keys; anything past this is not one. The cap bounds
/// the read so a huge file, or a symlink to an endless source, can't exhaust
/// the host process.
const MAX_THEME_BYTES: u64 = 64 * 1024;

/// The `current` symlink is deliberately left unresolved: the path is stable across themes and
/// switching theme rewrites the file behind it, so following it would pin the theme in place.
pub fn default_theme_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(THEME_FILE)
}

pub fn read_active_theme() -> Result<Theme, ThemeError> {
    read_theme(&default_theme_path())
}

pub fn read_theme(path: &Path) -> Result<Theme, ThemeError> {
    let text = read_theme_text(path)?;

    let table: Table =
        toml::from_str(&text).map_err(|cause| ThemeError::InvalidToml(path.to_owned(), cause))?;

    Ok(Theme {
        mode: read_mode(&table)?,
        colors: ThemeColors::from_table(&table)?,
    })
}

fn read_theme_text(path: &Path) -> Result<String, ThemeError> {
    let metadata = fs::metadata(path).map_err(|cause| read_error(path, cause))?;
    if !metadata.is_file() {
        return Err(ThemeError::Unreadable(
            path.to_owned(),
            io::Error::new(io::ErrorKind::InvalidInput, "not a regular file"),
        ));
    }
    if metadata.len() > MAX_THEME_BYTES {
        return Err(ThemeError::TooLarge(path.to_owned(), metadata.len()));
    }

    let file = fs::File::open(path).map_err(|cause| read_error(path, cause))?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_THEME_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|cause| ThemeError::Unreadable(path.to_owned(), cause))?;

    if bytes.len() as u64 > MAX_THEME_BYTES {
        return Err(ThemeError::TooLarge(path.to_owned(), bytes.len() as u64));
    }

    String::from_utf8(bytes).map_err(|cause| {
        ThemeError::Unreadable(
            path.to_owned(),
            io::Error::new(io::ErrorKind::InvalidData, cause),
        )
    })
}

fn read_error(path: &Path, cause: io::Error) -> ThemeError {
    if cause.kind() == io::ErrorKind::NotFound {
        ThemeError::NotFound(path.to_owned())
    } else {
        ThemeError::Unreadable(path.to_owned(), cause)
    }
}

fn read_mode(table: &Table) -> Result<Mode, ThemeError> {
    let value = table.get("mode").ok_or(ThemeError::MissingKey("mode"))?;
    let text = value
        .as_str()
        .ok_or_else(|| ThemeError::InvalidMode(value.to_string()))?;
    Mode::parse(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Rgb;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    fn read_fixture(name: &str) -> Result<Theme, ThemeError> {
        read_theme(&fixture(name))
    }

    #[test]
    fn reads_a_complete_dark_theme() {
        let theme = read_fixture("everforest.toml").unwrap();

        assert_eq!(theme.mode, Mode::Dark);
        assert_eq!(theme.colors.background, Rgb::parse("k", "#2d353b").unwrap());
        assert_eq!(theme.colors.foreground, Rgb::parse("k", "#d3c6aa").unwrap());
        assert_eq!(theme.colors.accent, Rgb::parse("k", "#7fbbb3").unwrap());
        assert_eq!(theme.colors.selection, Rgb::parse("k", "#3d484d").unwrap());
    }

    #[test]
    fn reads_a_light_theme() {
        let theme = read_fixture("flexoki-light.toml").unwrap();

        assert_eq!(theme.mode, Mode::Light);
        assert!(!theme.mode.is_dark());
    }

    /// `orange` and `brown` are outside the color contract: the fixture declares both, and reading
    /// it still succeeds, with nothing in `ThemeColors` carrying them.
    #[test]
    fn ignores_orange_and_brown() {
        let theme = read_fixture("everforest.toml").unwrap();

        assert_eq!(theme.colors.background, Rgb::parse("k", "#2d353b").unwrap());
    }

    #[test]
    fn ignores_keys_meant_for_other_applications() {
        let theme = read_fixture("with-extra-keys.toml").unwrap();

        assert_eq!(theme.colors.background, Rgb::parse("k", "#2d353b").unwrap());
    }

    #[test]
    fn tells_a_missing_theme_from_an_invalid_file() {
        let missing = read_fixture("does-not-exist.toml").unwrap_err();
        assert!(matches!(missing, ThemeError::NotFound(_)));

        let invalid = read_fixture("malformed.toml").unwrap_err();
        assert!(matches!(invalid, ThemeError::InvalidToml(_, _)));
    }

    #[test]
    fn the_invalid_toml_message_leaves_out_the_quoted_line() {
        let error = read_fixture("malformed.toml").unwrap_err();
        let message = error.to_string();

        assert!(message.contains("invalid TOML in"));
        assert!(
            !message.contains("[[["),
            "the message quoted the file: {message}"
        );
    }

    #[test]
    fn rejects_a_theme_larger_than_the_cap() {
        let error = read_fixture("oversized.toml").unwrap_err();

        assert!(matches!(error, ThemeError::TooLarge(_, size) if size > 64 * 1024));
    }

    #[test]
    fn rejects_a_path_that_is_not_a_regular_file() {
        let error = read_theme(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap_err();

        assert!(matches!(error, ThemeError::Unreadable(_, _)));
    }

    #[test]
    fn reports_the_missing_key() {
        let error = read_fixture("missing-background.toml").unwrap_err();
        assert!(matches!(error, ThemeError::MissingKey("background")));
    }

    #[test]
    fn reports_the_invalid_mode() {
        let error = read_fixture("invalid-mode.toml").unwrap_err();
        assert!(matches!(error, ThemeError::InvalidMode(value) if value == "sepia"));
    }

    #[test]
    fn reports_the_invalid_color_with_its_key() {
        let error = read_fixture("invalid-color.toml").unwrap_err();
        assert!(
            matches!(error, ThemeError::InvalidColor("accent", value) if value == "rgb(1e1e1e)")
        );
    }

    #[test]
    fn the_default_path_points_at_the_active_theme() {
        let path = default_theme_path();

        assert!(path.ends_with(THEME_FILE));
        if std::env::var_os("HOME").is_some() {
            assert!(path.is_absolute());
        }
    }
}
