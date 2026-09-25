//! API — the stable public contract: palettes of colors, basic in, extended and readable out.
//!
//! A library that depends on this crate gets its colors from a [`PaletteSeed`]: either the one
//! extracted from the active Omarchy theme, or one the application builds itself. [`extend`](PaletteSeed::extend)
//! turns that basic palette into a [`Palette`], the extended legible
//! palette: the semantic roles, the content that reads on each, and the tonal ramps to scale from.
//! The library builds no theme and picks no colors on the application's behalf beyond the readable
//! defaults a basic palette left unstated.

mod ffi;
mod palette;
mod theme;

pub use palette::{Palette, PaletteSeed};
pub use theme::ThemeWatch;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Role;
    use crate::theme::{Rgb, ThemeError};
    use std::path::{Path, PathBuf};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn reads_a_basic_palette_from_an_omarchy_lib_theme() {
        let basic = PaletteSeed::from_path(&fixture("everforest.toml")).unwrap();
        let palette = basic.extend();

        assert_eq!(
            palette.color(Role::Background),
            Rgb::parse("k", "#2d353b").unwrap()
        );
        assert_eq!(
            palette.color(Role::Primary),
            Rgb::parse("k", "#7fbbb3").unwrap()
        );
    }

    #[test]
    fn reports_a_missing_theme() {
        let error = PaletteSeed::from_path(&fixture("does-not-exist.toml")).unwrap_err();
        assert!(matches!(error, ThemeError::NotFound(_)));
    }

    #[test]
    fn infers_dark_from_a_dark_background() {
        let basic = PaletteSeed::new(
            Rgb::parse("k", "#111111").unwrap(),
            Rgb::parse("k", "#eeeeee").unwrap(),
            Rgb::parse("k", "#7fbbb3").unwrap(),
        );

        assert!(basic.extend().dark());
    }

    #[test]
    fn infers_light_from_a_light_background() {
        let basic = PaletteSeed::new(
            Rgb::parse("k", "#fafaf8").unwrap(),
            Rgb::parse("k", "#2a2a28").unwrap(),
            Rgb::parse("k", "#7fbbb3").unwrap(),
        );

        assert!(!basic.extend().dark());
    }

    #[test]
    fn follows_the_theme_mode_over_the_background_tone() {
        let basic = PaletteSeed::from_path(&fixture("mode-contradicts-tone.toml")).unwrap();

        assert!(!basic.extend().dark());
    }

    #[test]
    fn lets_the_application_override_the_inferred_mode() {
        let basic = PaletteSeed::new(
            Rgb::parse("k", "#111111").unwrap(),
            Rgb::parse("k", "#eeeeee").unwrap(),
            Rgb::parse("k", "#7fbbb3").unwrap(),
        );

        assert!(!basic.with_mode(crate::theme::Mode::Light).extend().dark());
    }

    #[test]
    fn keeps_every_content_readable_across_an_app_built_palette() {
        for background in ["#111111", "#fafaf8", "#5a5a5a"] {
            let basic = PaletteSeed::new(
                Rgb::parse("k", background).unwrap(),
                Rgb::parse(
                    "k",
                    if background == "#111111" {
                        "#eeeeee"
                    } else {
                        "#222222"
                    },
                )
                .unwrap(),
                Rgb::parse("k", "#7fbbb3").unwrap(),
            );
            let palette = basic.extend();

            for role in Role::ALL {
                let gap = crate::color::separation(palette.on(role), palette.color(role));
                assert!(
                    gap >= 40.0,
                    "{background} {role:?} separates content by only {gap:.1}"
                );
            }
        }
    }

    #[test]
    fn separates_derived_focus_and_disabled_from_their_neighbors() {
        let palette = PaletteSeed::new(
            Rgb::parse("k", "#111111").unwrap(),
            Rgb::parse("k", "#eeeeee").unwrap(),
            Rgb::parse("k", "#7fbbb3").unwrap(),
        )
        .extend();

        assert_ne!(palette.color(Role::Focus), palette.color(Role::Primary));
        assert_ne!(palette.color(Role::Disabled), palette.color(Role::Muted));
    }

    #[test]
    fn gives_every_role_a_ramp_that_spans_the_scale() {
        let palette = PaletteSeed::new(
            Rgb::parse("k", "#111111").unwrap(),
            Rgb::parse("k", "#eeeeee").unwrap(),
            Rgb::parse("k", "#7fbbb3").unwrap(),
        )
        .extend();

        for role in Role::ALL {
            assert_eq!(
                palette.ramp(role).tone(0),
                Rgb::parse("k", "#000000").unwrap()
            );
            assert_eq!(
                palette.ramp(role).tone(100),
                Rgb::parse("k", "#ffffff").unwrap()
            );
        }
    }

    #[test]
    fn keeps_content_readable_on_an_arbitrary_color() {
        let palette = PaletteSeed::new(
            Rgb::parse("k", "#111111").unwrap(),
            Rgb::parse("k", "#eeeeee").unwrap(),
            Rgb::parse("k", "#7fbbb3").unwrap(),
        )
        .extend();

        for ground in ["#7fbbb3", "#8f9aa8", "#e09d7f"] {
            let ground = Rgb::parse("k", ground).unwrap();
            let gap = crate::color::separation(palette.on_color(ground), ground);
            assert!(
                gap >= 40.0,
                "content on {ground} separates by only {gap:.1}"
            );
        }
    }
}
