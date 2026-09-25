//! Placing the raised and the recessed surface so both stand off the background and off each
//! other, and the line that marks the interface apart from its canvas.

use crate::theme::{Mode, Rgb, ThemeColors};

use super::ramp::{edge, shifted};
use super::tone::tone;

/// A card has to be noticed without looking for it; a well only has to read as another plane.
const RAISED_FROM_BACKGROUND: f64 = 6.0;
const RECESSED_FROM_BACKGROUND: f64 = 4.0;
const BETWEEN_SURFACES: f64 = 4.0;

/// A border only has to be noticed once it is looked for, so it stays under the recessed surface —
/// but a line reads worse than a plane at the same distance, so it cannot go much under it either.
const BORDER_FROM_BACKGROUND: f64 = 3.0;

#[derive(Clone, Copy)]
enum Side {
    Lighter,
    Darker,
}

pub(super) struct Surfaces {
    pub raised: Rgb,
    pub recessed: Rgb,
}

/// Themes declare `lighter_background` and `dark_background` around their own background, but the
/// names only hold on dark themes: on a light one both sit below the background, and which of them
/// is meant to be raised is told by distance, not by name.
pub(super) fn surfaces(colors: &ThemeColors, mode: Mode) -> Surfaces {
    let ground = tone(colors.background);
    match mode {
        Mode::Dark => around_the_background(ground, colors),
        Mode::Light => below_the_background(ground, colors),
    }
}

/// The border takes the background's own ramp to a fixed distance, rather than a declared key
/// pushed to a minimum like the surfaces: a key that already clears so short a distance would be
/// returned untouched, and the border would land on the raised surface instead of under it.
pub(super) fn border(colors: &ThemeColors, mode: Mode) -> Rgb {
    let ground = tone(colors.background);
    let lighter = matches!(mode, Mode::Dark);

    shifted(
        colors.background,
        edge(ground, BORDER_FROM_BACKGROUND, lighter),
    )
}

fn around_the_background(ground: f64, colors: &ThemeColors) -> Surfaces {
    let raised = pushed(
        colors.lighter_background,
        ground,
        Side::Lighter,
        RAISED_FROM_BACKGROUND,
    );

    // A background already at the bottom of the scale leaves no room for a darker plane, so the
    // well goes to the other side and stays beyond the card instead.
    let recessed = if ground >= RECESSED_FROM_BACKGROUND {
        pushed(
            colors.dark_background,
            ground,
            Side::Darker,
            RECESSED_FROM_BACKGROUND,
        )
    } else {
        pushed(
            colors.dark_background,
            tone(raised),
            Side::Lighter,
            BETWEEN_SURFACES,
        )
    };

    Surfaces { raised, recessed }
}

fn below_the_background(ground: f64, colors: &ThemeColors) -> Surfaces {
    let (nearer, farther) = by_distance(ground, colors);
    let raised = pushed(nearer, ground, Side::Darker, RAISED_FROM_BACKGROUND);
    let recessed = pushed(farther, tone(raised), Side::Darker, BETWEEN_SURFACES);

    Surfaces { raised, recessed }
}

fn by_distance(ground: f64, colors: &ThemeColors) -> (Rgb, Rgb) {
    let lighter = colors.lighter_background;
    let darker = colors.dark_background;
    if (ground - tone(lighter)).abs() <= (ground - tone(darker)).abs() {
        (lighter, darker)
    } else {
        (darker, lighter)
    }
}

/// Leaves `color` untouched when it already clears `minimum` from `reference` on `side` — the
/// theme leads as far as it serves — and moves it along its own ramp to the edge of that distance
/// when it does not.
fn pushed(color: Rgb, reference: f64, side: Side, minimum: f64) -> Rgb {
    let lighter = matches!(side, Side::Lighter);
    let current = tone(color);
    let clears = if lighter {
        current >= reference + minimum
    } else {
        current <= reference - minimum
    };

    if clears {
        return color;
    }

    shifted(color, edge(reference, minimum, lighter))
}
