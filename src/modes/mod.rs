//! MODES — the same theme, taken through a different treatment before COLOR resolves it.

mod accent;
mod high_contrast;
mod invert;
mod mono;
mod print;
mod sepia;
mod weights;

use crate::theme::{Rgb, Theme, ThemeColors};

/// A way of taking the theme's raw colors before [`Roles::resolve`](crate::color::Roles::resolve)
/// sees them. Each variant returns a full [`Theme`], so every guarantee COLOR already makes
/// (surfaces that stand off the background, content that stays in its ground's family, roles that
/// do not collide) is recomputed fresh from the transformed base — never duplicated here.
///
/// A treatment applies on top of the [`Mode`](crate::theme::Mode) a theme already resolves to; it
/// is not a peer of `Mode`, it is an operation that runs after one is settled. Three variants change
/// the resulting mode by design: [`Inverted`](Self::Inverted) restates the same theme for the
/// opposite ground — the point of it, not a side effect — and [`Print`](Self::Print) and
/// [`Sepia`](Self::Sepia) always force [`Light`](crate::theme::Mode::Light) because paper is light,
/// unrelated to polarity. The other three leave the mode untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Treatment {
    /// The theme as declared, untouched.
    Original,
    /// The same theme restated for the opposite ground, keeping its identity — not a color
    /// negative: hue and chroma survive, only the tone each color sits at moves. Flips the mode.
    Inverted,
    /// Every color pushed further from the background than it already stood.
    HighContrast,
    /// Every color desaturated: same tone, no hue.
    Mono,
    /// Forced onto a page: white behind, black in front, and every color as its own grey. Always
    /// resolves to [`Light`](crate::theme::Mode::Light), because the page is white.
    Print,
    /// Forced onto aged paper: a warm ground behind, dark brown in front, and every color a shade
    /// of that one brown. Always resolves to [`Light`](crate::theme::Mode::Light), because the
    /// paper is light.
    Sepia,
}

impl Treatment {
    pub const ALL: [Self; 6] = [
        Self::Original,
        Self::Inverted,
        Self::HighContrast,
        Self::Mono,
        Self::Print,
        Self::Sepia,
    ];
}

/// Takes `theme` through `treatment`, returning a full theme COLOR can resolve as usual.
pub fn represent(theme: &Theme, treatment: Treatment) -> Theme {
    match treatment {
        Treatment::Original => theme.clone(),
        Treatment::Inverted => invert::invert(theme),
        Treatment::HighContrast => high_contrast::amplify(theme),
        Treatment::Mono => mono::desaturate(theme),
        Treatment::Print => print::print(theme),
        Treatment::Sepia => sepia::sepia(theme),
    }
}

/// Applies `f` to every color the theme declares.
pub(super) fn map_colors(colors: &ThemeColors, f: impl Fn(Rgb) -> Rgb) -> ThemeColors {
    map_split(colors, &f, &f)
}

/// Applies `structure` to the colors that carry the page — the grounds, the text, and the surfaces
/// the roles rest on — and `accent` to the ones that accent it. The split a treatment needs
/// whenever the two cannot take the same transform.
pub(super) fn map_split(
    colors: &ThemeColors,
    structure: impl Fn(Rgb) -> Rgb,
    accent: impl Fn(Rgb) -> Rgb,
) -> ThemeColors {
    ThemeColors {
        accent: accent(colors.accent),
        selection: structure(colors.selection),
        muted: structure(colors.muted),

        background: structure(colors.background),
        dark_background: structure(colors.dark_background),
        darker_background: structure(colors.darker_background),
        lighter_background: structure(colors.lighter_background),

        foreground: structure(colors.foreground),
        dark_foreground: structure(colors.dark_foreground),
        light_foreground: structure(colors.light_foreground),
        bright_foreground: structure(colors.bright_foreground),

        red: accent(colors.red),
        dark_red: accent(colors.dark_red),
        bright_red: accent(colors.bright_red),

        yellow: accent(colors.yellow),
        dark_yellow: accent(colors.dark_yellow),
        bright_yellow: accent(colors.bright_yellow),

        green: accent(colors.green),
        dark_green: accent(colors.dark_green),
        bright_green: accent(colors.bright_green),

        cyan: accent(colors.cyan),
        dark_cyan: accent(colors.dark_cyan),
        bright_cyan: accent(colors.bright_cyan),

        blue: accent(colors.blue),
        dark_blue: accent(colors.dark_blue),
        bright_blue: accent(colors.bright_blue),

        magenta: accent(colors.magenta),
        dark_magenta: accent(colors.dark_magenta),
        bright_magenta: accent(colors.bright_magenta),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::PaletteSeed;
    use crate::color::{Level, Role, separation};
    use std::path::{Path, PathBuf};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    /// Every guarantee COLOR already makes must survive every treatment: this is what makes
    /// "transform the theme, reuse Roles::resolve" a safe design instead of a hopeful one. Goes
    /// through `Palette::represented`, the seam every caller (Rust and the C bridge alike) shares,
    /// instead of calling `represent`/`Roles::resolve` directly.
    #[test]
    fn every_treatment_still_resolves_a_coherent_palette() {
        for fixture_name in ["everforest.toml", "flexoki-light.toml"] {
            let original = PaletteSeed::from_path(&fixture(fixture_name))
                .unwrap()
                .extend();

            for treatment in Treatment::ALL {
                let palette = original.represented(treatment);

                for role in Role::ALL {
                    let ground = palette.color(role);
                    let main = palette.content(role, Level::Primary);
                    let gap = separation(main, ground);
                    assert!(
                        gap >= 40.0,
                        "{fixture_name} {treatment:?} {role:?}: content separates by only {gap:.1}"
                    );
                }
            }
        }
    }

    #[test]
    fn original_changes_nothing() {
        let palette = PaletteSeed::from_path(&fixture("everforest.toml"))
            .unwrap()
            .extend();

        assert_eq!(palette.represented(Treatment::Original), palette);
    }

    /// `represented` always starts over from the palette's own source: taking a treatment of a
    /// treatment is the same as taking the second treatment directly, never a stack of the two.
    #[test]
    fn represented_never_stacks() {
        let palette = PaletteSeed::from_path(&fixture("everforest.toml"))
            .unwrap()
            .extend();

        assert_eq!(
            palette
                .represented(Treatment::Inverted)
                .represented(Treatment::HighContrast),
            palette.represented(Treatment::HighContrast)
        );
    }
}
