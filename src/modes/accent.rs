//! Where an accent belongs on a given ground, shared by the treatments that move one.

use crate::color::{Ramp, tone_of};
use crate::theme::{Mode, Rgb};

/// The tone at which an accent reads against each ground: a light page carries its accents darker
/// than itself, a dark page carries them lighter.
const ON_LIGHT: f64 = 42.0;
const ON_DARK: f64 = 78.0;

/// How much of an accent's own distance from that tone survives the move — enough that a theme's
/// accents stay told apart from one another, little enough that none is sent back to an extreme.
const SPREAD: f64 = 0.35;

fn anchor(ground: Mode) -> f64 {
    if ground.is_dark() { ON_DARK } else { ON_LIGHT }
}

/// Restates `color` — written for a theme standing on `was` — at the tone an accent takes on
/// `ground`, keeping its hue and chroma.
///
/// Carrying its tone across untouched is what leaves a theme's accent unreadable once the ground
/// moves out from under it, and reflecting that tone is worse still: an accent's separation from a
/// near-black ground is nearly the whole scale, and meeting it against a near-white one lands at
/// the far extreme, bringing the brightest accent back almost black.
pub(super) fn restated(color: Rgb, was: Mode, ground: Mode) -> Rgb {
    let tone = anchor(ground) + (tone_of(color) - anchor(was)) * SPREAD;
    Ramp::of(color).tone(tone.clamp(0.0, 100.0).round() as u8)
}

/// How far an accent has to stand from the ground it sits on. The same distance COLOR demands of
/// content over a ground, because an application uses its accent as content too — the reference
/// application draws its own title in it.
const CLEAR_OF_GROUND: f64 = 40.0;

/// `color` as the theme wrote it, moved only as far as the new ground forces it — and not at all
/// when it already stands clear.
///
/// The theme keeps its identity across a change of polarity this way: what changes is which side of
/// the ground a color sits on, not the color. Restating every accent at a tone chosen for the new
/// ground moves colors that had no need to move.
pub(super) fn kept(color: Rgb, ground: Rgb) -> Rgb {
    let ground_tone = tone_of(ground);
    let tone = tone_of(color);

    if (tone - ground_tone).abs() >= CLEAR_OF_GROUND {
        return color;
    }

    let outward = if ground_tone < 50.0 {
        ground_tone + CLEAR_OF_GROUND
    } else {
        ground_tone - CLEAR_OF_GROUND
    };

    Ramp::of(color).tone(outward.clamp(0.0, 100.0).round() as u8)
}

/// A surface of the theme taken to a tint light enough to sit on a printed page, keeping its hue.
pub(super) fn tinted(color: Rgb, tone: u8) -> Rgb {
    Ramp::of(color).tone(tone)
}
