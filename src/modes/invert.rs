//! Inverted: the same theme read the other way around.

use crate::color::{Ramp, tone_of};
use crate::theme::{Rgb, Theme};

use super::accent::kept;
use super::map_split;

/// Reflects `color`'s tone around the middle of the scale, keeping its hue and chroma — the same
/// separation two colors have today, their reflections still have, so nothing COLOR already
/// guarantees breaks under this transform.
fn reflected(color: Rgb) -> Rgb {
    Ramp::of(color).tone((100.0 - tone_of(color)).round() as u8)
}

/// A theme read inverted also has to flip which ground it is built on — a dark theme's colors
/// reflected become a light theme's colors, and the surfaces derived from them need to know that.
pub(super) fn invert(theme: &Theme) -> Theme {
    let ground = reflected(theme.colors.background);

    Theme {
        mode: theme.mode.flipped(),
        colors: map_split(&theme.colors, reflected, |color| kept(color, ground)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{hue_gap, separation};
    use crate::theme::{Mode, read_theme};
    use std::path::Path;

    fn fixture(name: &str) -> Theme {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        read_theme(&path).unwrap()
    }

    #[test]
    fn flips_the_theme_mode() {
        let dark = fixture("everforest.toml");
        assert_eq!(invert(&dark).mode, Mode::Light);

        let light = fixture("flexoki-light.toml");
        assert_eq!(invert(&light).mode, Mode::Dark);
    }

    #[test]
    fn preserves_the_separation_between_two_colors() {
        let theme = fixture("everforest.toml");
        let inverted = invert(&theme);

        let original_gap = separation(theme.colors.background, theme.colors.foreground);
        let inverted_gap = separation(inverted.colors.background, inverted.colors.foreground);

        assert!((original_gap - inverted_gap).abs() < 1.0);
    }

    #[test]
    fn keeps_an_accent_in_its_own_hue() {
        let theme = fixture("everforest.toml");
        let inverted = invert(&theme);

        assert!(hue_gap(theme.colors.accent, inverted.colors.accent) < 10.0);
        assert!(hue_gap(theme.colors.red, inverted.colors.red) < 10.0);
    }

    /// An accent that already stands clear of the ground the flip gives it has nothing to gain from
    /// being moved, and the theme keeps its identity by leaving it alone.
    #[test]
    fn leaves_an_accent_that_already_stands_clear() {
        let theme = fixture("flexoki-light.toml");
        let inverted = invert(&theme);

        assert_eq!(inverted.colors.red, theme.colors.red);
    }

    /// The defect this transform was rewritten for: a bright accent came back almost black, because
    /// reflecting it reproduced against the new ground the whole separation it held from the old.
    #[test]
    fn keeps_an_accent_off_the_extremes() {
        for name in ["everforest.toml", "flexoki-light.toml"] {
            let inverted = invert(&fixture(name));

            for accent in [inverted.colors.accent, inverted.colors.red] {
                let tone = tone_of(accent);
                assert!(
                    (20.0..=90.0).contains(&tone),
                    "{name}: an accent landed at tone {tone}"
                );
            }
        }
    }

    #[test]
    fn reflects_a_dark_ground_into_a_light_one() {
        let theme = fixture("everforest.toml");
        let inverted = invert(&theme);

        assert!(tone_of(inverted.colors.background) > tone_of(theme.colors.background));
    }
}
