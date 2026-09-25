//! Sepia: forced onto aged paper — a warm ground behind, dark brown ink in front, and every color a
//! shade of the same brown.

use crate::theme::{Mode, Rgb, Theme, ThemeColors};

use super::accent::restated;
use super::weights::{Weights, at};

/// The one hue every color of the page is taken from. Fixed, so the treatment reads the same on
/// every theme: it belongs to the paper, not to the theme it is applied to.
const SEPIA: Rgb = Rgb {
    red: 112,
    green: 66,
    blue: 20,
};

/// The tones of the page: the ground close to the top of the scale, the ink close to the bottom,
/// and the two surfaces the theme brought settled between them.
const PAPER: [f64; 4] = [94.0, 97.0, 91.0, 88.0];
const INK: [f64; 4] = [18.0, 12.0, 18.0, 24.0];
const SELECTION_TONE: f64 = 86.0;
const MUTED_TONE: f64 = 78.0;

/// A page of one warm hue: what tells two colors apart is where each falls between the paper and
/// the ink, so each keeps the tone its own weight gives it and is moved only when another already
/// holds it.
///
/// Every color is restated for a light ground first, as on a printed page, so an accent written for
/// a dark screen does not arrive washed out against the paper.
pub(super) fn sepia(theme: &Theme) -> Theme {
    let colors = &theme.colors;
    let was = theme.mode;
    let ink = |color: Rgb| restated(color, was, Mode::Light);

    let mut weights = Weights::new(paint);
    let accent = weights.place(ink(colors.accent));
    let mut family =
        |color: Rgb, dark: Rgb, bright: Rgb| weights.family(ink(color), ink(dark), ink(bright));
    let red = family(colors.red, colors.dark_red, colors.bright_red);
    let green = family(colors.green, colors.dark_green, colors.bright_green);
    let yellow = family(colors.yellow, colors.dark_yellow, colors.bright_yellow);
    let blue = family(colors.blue, colors.dark_blue, colors.bright_blue);
    let cyan = family(colors.cyan, colors.dark_cyan, colors.bright_cyan);
    let magenta = family(colors.magenta, colors.dark_magenta, colors.bright_magenta);

    Theme {
        mode: Mode::Light,
        colors: ThemeColors {
            background: shade(PAPER[0]),
            lighter_background: shade(PAPER[1]),
            dark_background: shade(PAPER[2]),
            darker_background: shade(PAPER[3]),
            foreground: shade(INK[0]),
            bright_foreground: shade(INK[1]),
            light_foreground: shade(INK[2]),
            dark_foreground: shade(INK[3]),
            accent,
            selection: shade(SELECTION_TONE),
            muted: shade(MUTED_TONE),
            red: red.0,
            dark_red: red.1,
            bright_red: red.2,
            yellow: yellow.0,
            dark_yellow: yellow.1,
            bright_yellow: yellow.2,
            green: green.0,
            dark_green: green.1,
            bright_green: green.2,
            cyan: cyan.0,
            dark_cyan: cyan.1,
            bright_cyan: cyan.2,
            blue: blue.0,
            dark_blue: blue.1,
            bright_blue: blue.2,
            magenta: magenta.0,
            dark_magenta: magenta.1,
            bright_magenta: magenta.2,
        },
    }
}

fn shade(tone: f64) -> Rgb {
    at(SEPIA, tone)
}

/// Whatever the theme's color was, only the tone it was given survives.
fn paint(_: Rgb, tone: f64) -> Rgb {
    shade(tone)
}

#[cfg(test)]
mod tests {
    use super::super::weights::APART;
    use super::*;
    use crate::color::{hue_gap, separation, tone_of};
    use crate::theme::read_theme;
    use std::path::Path;

    fn fixture(name: &str) -> Theme {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        read_theme(&path).unwrap()
    }

    fn palette(colors: &ThemeColors) -> [Rgb; 25] {
        [
            colors.background,
            colors.lighter_background,
            colors.dark_background,
            colors.darker_background,
            colors.foreground,
            colors.bright_foreground,
            colors.light_foreground,
            colors.dark_foreground,
            colors.accent,
            colors.selection,
            colors.muted,
            colors.red,
            colors.dark_red,
            colors.bright_red,
            colors.yellow,
            colors.green,
            colors.cyan,
            colors.blue,
            colors.magenta,
            colors.dark_green,
            colors.bright_green,
            colors.dark_blue,
            colors.bright_blue,
            colors.dark_cyan,
            colors.bright_magenta,
        ]
    }

    #[test]
    fn forces_a_light_page_whatever_the_theme_was() {
        for name in ["everforest.toml", "flexoki-light.toml"] {
            let page = sepia(&fixture(name));

            assert_eq!(page.mode, Mode::Light, "{name}");
            let gap = separation(page.colors.foreground, page.colors.background);
            assert!(
                gap >= 40.0,
                "{name}: the ink separates from the paper by {gap:.1}"
            );
        }
    }

    #[test]
    fn takes_every_color_from_the_one_hue() {
        let page = sepia(&fixture("everforest.toml"));

        for color in palette(&page.colors) {
            let drift = hue_gap(color, SEPIA);
            assert!(drift < 15.0, "{color} drifts {drift:.1} from the sepia");
        }
    }

    /// The defect `Mono` still has: with a single hue only weight is left to tell two colors apart.
    #[test]
    fn tells_apart_the_colors_that_only_their_hue_told_apart() {
        for name in ["everforest.toml", "flexoki-light.toml"] {
            let colors = sepia(&fixture(name)).colors;
            let mut tones = [
                colors.accent,
                colors.red,
                colors.green,
                colors.yellow,
                colors.blue,
                colors.cyan,
                colors.magenta,
            ]
            .map(tone_of);
            tones.sort_by(|one, other| one.partial_cmp(other).expect("a tone is never NaN"));

            for pair in tones.windows(2) {
                let gap = pair[1] - pair[0];
                assert!(gap >= APART - 1.0, "{name}: two colors sit {gap:.1} apart");
            }
        }
    }

    #[test]
    fn leaves_the_colors_readable_on_the_paper() {
        let colors = sepia(&fixture("everforest.toml")).colors;

        for ink in [colors.accent, colors.red, colors.blue] {
            let gap = separation(ink, colors.background);
            assert!(
                gap >= 40.0,
                "an accent separates from the paper by {gap:.1}"
            );
        }
    }
}
