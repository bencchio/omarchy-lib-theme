//! Choosing the colors that sit on top of another one and stay readable.

use crate::theme::{Rgb, ThemeColors};

use super::ramp::shifted;
use super::tone::{separation, tone};

/// Material keeps content legible when it stands at least this far from its ground in tone.
const MIN_SEPARATION: f64 = 40.0;
/// Secondary content steps back from the main one without leaving the readable range; disabled
/// content steps back far enough to read as switched off, and still stands apart from its ground.
const SECONDARY_SEPARATION: f64 = 28.0;
const DISABLED_SEPARATION: f64 = 18.0;

/// How much weight a piece of content carries on its ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    /// Body text and anything the reader is meant to act on.
    Primary,
    /// Supporting text, such as a caption under a title.
    Secondary,
    /// Content that cannot be interacted with.
    Disabled,
}

impl Level {
    pub const ALL: [Self; 3] = [Self::Primary, Self::Secondary, Self::Disabled];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Levels {
    primary: Rgb,
    secondary: Rgb,
    disabled: Rgb,
}

impl Levels {
    pub(super) fn at(self, level: Level) -> Rgb {
        match level {
            Level::Primary => self.primary,
            Level::Secondary => self.secondary,
            Level::Disabled => self.disabled,
        }
    }
}

/// The three weights of content that read on `ground`.
///
/// The lighter weights are taken off the main one along its own ramp, so all three keep the same
/// color and differ only in how far they stand from their ground.
pub(super) fn levels(ground: Rgb, colors: &ThemeColors) -> Levels {
    let primary = on(ground, colors);

    Levels {
        primary,
        secondary: stepped_back(primary, ground, SECONDARY_SEPARATION),
        disabled: stepped_back(primary, ground, DISABLED_SEPARATION),
    }
}

/// Picks the first foreground the theme offers that stands far enough from `ground`.
///
/// The candidates are tried in the order the theme means them, so a theme's own `foreground` wins
/// whenever it already reads; only when none of them separates enough does this fall back to the
/// end of the ground's own tonal ramp, which always does.
pub(super) fn on(ground: Rgb, colors: &ThemeColors) -> Rgb {
    let candidates = [
        colors.foreground,
        colors.bright_foreground,
        colors.light_foreground,
        colors.dark_foreground,
    ];

    candidates
        .into_iter()
        .find(|candidate| separation(*candidate, ground) >= MIN_SEPARATION)
        .unwrap_or_else(|| own_family(ground))
}

fn stepped_back(content: Rgb, ground: Rgb, separation: f64) -> Rgb {
    let ground_tone = tone(ground);
    let target = if tone(content) > ground_tone {
        ground_tone + separation
    } else {
        ground_tone - separation
    };

    shifted(content, target)
}

/// The ramp's own ends are always pure black or white, so reaching for them would trade the "no
/// candidate reads" problem for a new one: content that has no relation to its ground's color at
/// all. Stopping at the minimum separation instead keeps the ground's hue and chroma, which the
/// ramp preserves everywhere short of its two achromatic ends.
fn own_family(ground: Rgb) -> Rgb {
    let ground_tone = tone(ground);
    let lighter = ground_tone < 50.0;

    shifted(
        ground,
        super::ramp::edge(ground_tone, MIN_SEPARATION, lighter),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::read_theme;
    use std::path::Path;

    fn theme_colors(name: &str) -> ThemeColors {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        read_theme(&path).unwrap().colors
    }

    #[test]
    fn prefers_the_theme_own_foreground_when_it_already_reads() {
        let colors = theme_colors("everforest.toml");

        assert_eq!(on(colors.background, &colors), colors.foreground);
    }

    #[test]
    fn always_separates_enough_from_its_ground() {
        for fixture in ["everforest.toml", "flexoki-light.toml"] {
            let colors = theme_colors(fixture);

            for ground in [
                colors.background,
                colors.lighter_background,
                colors.dark_background,
                colors.accent,
                colors.selection,
                colors.muted,
                colors.red,
            ] {
                let content = on(ground, &colors);
                assert!(
                    separation(content, ground) >= MIN_SEPARATION,
                    "{fixture}: {content} on {ground} separates by {:.1}",
                    separation(content, ground)
                );
            }
        }
    }

    #[test]
    fn falls_back_within_reach_of_the_ground_when_no_foreground_reads() {
        let mut colors = theme_colors("everforest.toml");
        let ground = colors.background;
        for candidate in [
            &mut colors.foreground,
            &mut colors.bright_foreground,
            &mut colors.light_foreground,
            &mut colors.dark_foreground,
        ] {
            *candidate = ground;
        }

        let content = on(ground, &colors);

        assert!(separation(content, ground) >= MIN_SEPARATION);
        assert!(tone(content) > tone(ground));
    }

    #[test]
    fn keeps_the_ground_hue_when_no_theme_foreground_reads() {
        let colors = theme_colors("no-reading-foreground.toml");
        let ground = colors.background;

        let content = on(ground, &colors);

        assert!(separation(content, ground) >= MIN_SEPARATION);
        assert!(
            super::super::tone::hue_gap(content, ground) < 15.0,
            "invented content should stay in the ground's own color family"
        );
    }

    #[test]
    fn no_longer_falls_to_an_achromatic_extreme_on_a_chromatic_ground() {
        let colors = theme_colors("no-reading-foreground.toml");
        let content = on(colors.accent, &colors);

        assert!(
            super::super::tone::chroma(content) > 5.0,
            "content {content} on {} lost the ground's hue",
            colors.accent
        );
    }

    #[test]
    fn does_not_force_hue_onto_a_low_chroma_ground() {
        let mut colors = theme_colors("everforest.toml");
        colors.background = Rgb::parse("k", "#4a4a4a").unwrap();
        for candidate in [
            &mut colors.foreground,
            &mut colors.bright_foreground,
            &mut colors.light_foreground,
            &mut colors.dark_foreground,
        ] {
            *candidate = colors.background;
        }

        let content = on(colors.background, &colors);

        assert!(separation(content, colors.background) >= MIN_SEPARATION);
        assert!(super::super::tone::chroma(content) < 5.0);
    }
}
