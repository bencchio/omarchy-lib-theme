//! High Contrast: the same relationships the theme already has, pushed further apart.

use crate::color::{Ramp, tone_of};
use crate::theme::{Rgb, Theme};

use super::map_colors;

/// Scales up whatever distance a color already keeps from the background, instead of jumping to
/// an extreme — a color barely off the background moves a little, one already far off moves more,
/// and the background itself (distance zero) does not move at all.
const CONTRAST_FACTOR: f64 = 1.6;

fn pushed_apart(color: Rgb, background_tone: f64) -> Rgb {
    let distance = tone_of(color) - background_tone;
    if distance == 0.0 {
        return color;
    }

    let target = (background_tone + distance * CONTRAST_FACTOR).clamp(0.0, 100.0);
    Ramp::of(color).tone(target.round() as u8)
}

pub(super) fn amplify(theme: &Theme) -> Theme {
    let background_tone = tone_of(theme.colors.background);

    Theme {
        mode: theme.mode,
        colors: map_colors(&theme.colors, |color| pushed_apart(color, background_tone)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::separation;
    use crate::theme::read_theme;
    use std::path::Path;

    fn fixture(name: &str) -> Theme {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        read_theme(&path).unwrap()
    }

    #[test]
    fn leaves_the_background_where_it_is() {
        let theme = fixture("everforest.toml");
        let amplified = amplify(&theme);

        assert_eq!(amplified.colors.background, theme.colors.background);
    }

    #[test]
    fn widens_the_gap_to_the_background() {
        let theme = fixture("everforest.toml");
        let amplified = amplify(&theme);

        let original_gap = separation(theme.colors.foreground, theme.colors.background);
        let amplified_gap = separation(amplified.colors.foreground, amplified.colors.background);

        assert!(
            amplified_gap >= original_gap,
            "amplified gap {amplified_gap:.1} did not widen from {original_gap:.1}"
        );
    }

    #[test]
    fn keeps_the_theme_mode() {
        let theme = fixture("everforest.toml");
        assert_eq!(amplify(&theme).mode, theme.mode);
    }
}
