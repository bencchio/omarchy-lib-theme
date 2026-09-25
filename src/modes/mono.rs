//! Mono: the same theme, with every color's hue removed.

use crate::color::desaturate as flatten;
use crate::theme::Theme;

use super::map_colors;

pub(super) fn desaturate(theme: &Theme) -> Theme {
    Theme {
        mode: theme.mode,
        colors: map_colors(&theme.colors, flatten),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::chroma_of;
    use crate::theme::read_theme;
    use std::path::Path;

    fn fixture(name: &str) -> Theme {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        read_theme(&path).unwrap()
    }

    #[test]
    fn drops_the_hue_of_every_declared_color() {
        let theme = fixture("everforest.toml");
        let mono = desaturate(&theme);

        for (original, flattened) in [
            (theme.colors.accent, mono.colors.accent),
            (theme.colors.red, mono.colors.red),
            (theme.colors.selection, mono.colors.selection),
        ] {
            assert!(
                chroma_of(flattened) < chroma_of(original) / 4.0,
                "{flattened} still carries {:.1} chroma from {original}",
                chroma_of(flattened)
            );
        }
    }

    #[test]
    fn keeps_the_theme_mode() {
        let theme = fixture("everforest.toml");
        assert_eq!(desaturate(&theme).mode, theme.mode);
    }
}
