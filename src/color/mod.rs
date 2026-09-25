//! COLOR — turns the theme's keys into the roles applications ask for, and into tonal ramps.

mod content;
pub(crate) mod deposit;
mod ramp;
mod role;
mod surface;
mod tone;
mod variants;

pub use content::Level;
pub use deposit::{Deposit, State, Styled};
pub use ramp::Ramp;
pub use role::Role;

use crate::theme::{Rgb, Theme, ThemeColors};

/// Perceptual lightness of a color, 0 (black) to 100 (white).
pub fn tone_of(color: Rgb) -> f64 {
    tone::tone(color)
}

/// How far two colors stand apart in perceptual lightness.
///
/// Content needs roughly 40 to stay readable on its ground; [`Roles::on`] already guarantees that,
/// so reach for this when pairing colors the roles do not cover.
pub fn separation(one: Rgb, other: Rgb) -> f64 {
    tone::separation(one, other)
}

/// How saturated a color reads, 0 (gray) upward.
///
/// A ground below roughly 12 has no hue worth keeping content in family with.
pub fn chroma_of(color: Rgb) -> f64 {
    tone::chroma(color)
}

/// How far two colors stand apart in hue, 0 to 180. Only meaningful when both colors carry some
/// [`chroma_of`]; a near-gray color has no hue to compare.
pub fn hue_gap(one: Rgb, other: Rgb) -> f64 {
    tone::hue_gap(one, other)
}

/// Content that stays readable on `color`, whatever it is.
///
/// The theme's own foregrounds win whenever one already reads; otherwise the color's own ramp
/// supplies a content that keeps its hue. This is the same guarantee [`Roles::on`] makes, lifted
/// off the fixed roles so a caller can highlight or mark with any color it chose.
pub fn on(color: Rgb, colors: &ThemeColors) -> Rgb {
    content::on(color, colors)
}

/// The same color at zero chroma: its tone unchanged, its hue meaningless once chroma is gone.
pub fn desaturate(color: Rgb) -> Rgb {
    tone::desaturate(color)
}

/// Every role resolved against one theme, each with the content colors that read on it.
///
/// A theme leads as far as it serves: its own keys are kept whenever they already stand apart, and
/// completed from the theme's own colors when they do not — a surface that matches the background
/// leaves nothing to draw a card with, and roles that resolve to one color leave a focused element
/// looking like a primary one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roles {
    background: Pair,
    surface: Pair,
    surface_variant: Pair,
    border: Pair,
    primary: Pair,
    selection: Pair,
    muted: Pair,
    error: Pair,
    focus: Pair,
    disabled: Pair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pair {
    color: Rgb,
    content: content::Levels,
}

impl Roles {
    pub fn resolve(theme: &Theme) -> Self {
        let colors = &theme.colors;
        let pair = |color: Rgb| Pair {
            color,
            content: content::levels(color, colors),
        };
        let surfaces = surface::surfaces(colors, theme.mode);

        Self {
            background: pair(colors.background),
            surface: pair(surfaces.raised),
            surface_variant: pair(surfaces.recessed),
            border: pair(surface::border(colors, theme.mode)),
            primary: pair(colors.accent),
            selection: pair(colors.selection),
            muted: pair(colors.muted),
            error: pair(colors.red),
            focus: pair(variants::focus(colors.accent, colors.background)),
            disabled: pair(variants::disabled(colors.muted, colors.background)),
        }
    }

    pub fn color(&self, role: Role) -> Rgb {
        self.pair(role).color
    }

    /// The main content color that stays readable on top of `role`.
    pub fn on(&self, role: Role) -> Rgb {
        self.content(role, Level::Primary)
    }

    /// The content color of a given weight that reads on top of `role`.
    pub fn content(&self, role: Role, level: Level) -> Rgb {
        self.pair(role).content.at(level)
    }

    pub fn ramp(&self, role: Role) -> Ramp {
        Ramp::of(self.color(role))
    }

    fn pair(&self, role: Role) -> Pair {
        match role {
            Role::Background => self.background,
            Role::Elevated => self.surface,
            Role::Recessed => self.surface_variant,
            Role::Border => self.border,
            Role::Primary => self.primary,
            Role::Selection => self.selection,
            Role::Muted => self.muted,
            Role::Error => self.error,
            Role::Focus => self.focus,
            Role::Disabled => self.disabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Palette, PaletteSeed};
    use crate::theme::read_theme;
    use std::path::{Path, PathBuf};
    use tone::separation;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    /// Resolved through the seam every public caller shares — `PaletteSeed::from_theme`, lossless
    /// — instead of calling `Roles::resolve` directly, so these tests exercise the same path Rust
    /// and the C bridge both read from.
    fn palette(name: &str) -> (Theme, Palette) {
        let theme = read_theme(&fixture(name)).unwrap();
        let palette = PaletteSeed::from_theme(&theme).extend();
        (theme, palette)
    }

    #[test]
    fn maps_the_roles_the_theme_backs_directly() {
        let (theme, palette) = palette("everforest.toml");
        let colors = &theme.colors;

        assert_eq!(palette.color(Role::Background), colors.background);
        assert_eq!(palette.color(Role::Primary), colors.accent);
        assert_eq!(palette.color(Role::Selection), colors.selection);
        assert_eq!(palette.color(Role::Muted), colors.muted);
        assert_eq!(palette.color(Role::Error), colors.red);
    }

    #[test]
    fn keeps_every_content_color_readable_in_both_modes() {
        for fixture in ["everforest.toml", "flexoki-light.toml"] {
            let (_, palette) = palette(fixture);

            for role in Role::ALL {
                let separation = separation(palette.on(role), palette.color(role));
                assert!(
                    separation >= 40.0,
                    "{fixture}: {role:?} separates content by only {separation:.1}"
                );
            }
        }
    }

    /// The defect the seam unification fixed: a theme's own `bright_foreground` clears the 40-unit
    /// minimum on the accent, but `foreground` alone does not — so a caller that only sees
    /// `foreground` (the old, lossy `PaletteSeed`) invents a color instead of using it. Shaped
    /// after `lupine`, the theme the mismatch was first measured on.
    #[test]
    fn uses_a_foreground_variant_the_theme_declares_when_the_default_one_does_not_read() {
        let (theme, palette) = palette("foreground-variant-reads.toml");

        assert!(separation(theme.colors.foreground, theme.colors.accent) < 40.0);
        assert!(separation(theme.colors.bright_foreground, theme.colors.accent) >= 40.0);
        assert_eq!(palette.on(Role::Primary), theme.colors.bright_foreground);
    }

    #[test]
    fn stands_both_surfaces_off_the_background_and_off_each_other() {
        for fixture in [
            "everforest.toml",
            "flexoki-light.toml",
            "flat-surface.toml",
            "light-on-white.toml",
        ] {
            let (_, palette) = palette(fixture);
            let background = palette.color(Role::Background);
            let surface = palette.color(Role::Elevated);
            let variant = palette.color(Role::Recessed);

            for (name, color) in [("surface", surface), ("surface variant", variant)] {
                let gap = separation(color, background);
                assert!(
                    gap >= 4.0,
                    "{fixture}: {name} sits {gap:.1} off the background"
                );
            }
            let between = separation(surface, variant);
            assert!(
                between >= 4.0,
                "{fixture}: the two surfaces sit {between:.1} apart"
            );
        }
    }

    #[test]
    fn marks_the_border_off_the_background_without_reaching_the_card() {
        for fixture in [
            "everforest.toml",
            "flexoki-light.toml",
            "flat-surface.toml",
            "light-on-white.toml",
        ] {
            let (_, palette) = palette(fixture);
            let background = palette.color(Role::Background);
            let border = palette.color(Role::Border);
            let surface = palette.color(Role::Elevated);

            let gap = separation(border, background);
            assert!(
                gap >= 3.0,
                "{fixture}: the border sits {gap:.1} off the background"
            );

            let card = separation(surface, background);
            assert!(
                gap < card,
                "{fixture}: the border stands {gap:.1} off the background, as far as the card at {card:.1}"
            );

            assert_ne!(
                border, background,
                "{fixture}: the border melts into the background"
            );
            assert_ne!(border, surface, "{fixture}: the border lands on the card");
        }
    }

    #[test]
    fn raises_the_card_off_a_light_background_instead_of_lightening_it() {
        let (_, palette) = palette("flexoki-light.toml");

        let background = tone_of(palette.color(Role::Background));
        let surface = tone_of(palette.color(Role::Elevated));
        let variant = tone_of(palette.color(Role::Recessed));

        assert!(
            surface < background,
            "the card should not chase a light background"
        );
        assert!(
            variant < surface,
            "the well should sit deeper than the card"
        );
    }

    #[test]
    fn completes_a_surface_the_theme_leaves_flat() {
        let (theme, palette) = palette("flat-surface.toml");

        assert_ne!(palette.color(Role::Elevated), theme.colors.background);
    }

    #[test]
    fn tells_apart_the_roles_a_theme_backs_with_one_color() {
        for fixture in ["everforest.toml", "flexoki-light.toml"] {
            let (_, palette) = palette(fixture);

            for (one, other) in [(Role::Focus, Role::Primary), (Role::Disabled, Role::Muted)] {
                let gap = separation(palette.color(one), palette.color(other));
                assert!(
                    gap >= 8.0,
                    "{fixture}: {one:?} sits {gap:.1} from {other:?}"
                );
            }
        }
    }

    #[test]
    fn steps_every_content_level_back_from_the_one_before() {
        for fixture in ["everforest.toml", "flexoki-light.toml"] {
            let (_, palette) = palette(fixture);

            for role in Role::ALL {
                let ground = palette.color(role);
                let gaps: Vec<f64> = Level::ALL
                    .iter()
                    .map(|level| separation(palette.content(role, *level), ground))
                    .collect();

                assert!(
                    gaps[0] > gaps[1] && gaps[1] > gaps[2],
                    "{fixture}: {role:?} does not step back its content: {gaps:?}"
                );
                assert!(
                    gaps[2] >= 15.0,
                    "{fixture}: {role:?} loses its disabled content at {:.1}",
                    gaps[2]
                );
            }
        }
    }

    #[test]
    fn gives_every_role_a_ramp_that_spans_the_scale() {
        let (_, palette) = palette("everforest.toml");

        for role in Role::ALL {
            let ramp = palette.ramp(role);
            assert_eq!(ramp.tone(0), Rgb::parse("k", "#000000").unwrap());
            assert_eq!(ramp.tone(100), Rgb::parse("k", "#ffffff").unwrap());
        }
    }
}
