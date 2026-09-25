//! Pulling apart the roles a theme backs with a single color.

use crate::theme::Rgb;

use super::ramp::shifted;
use super::tone::tone;

/// Focus and primary both come from the theme's accent, and a keyboard-driven interface is unusable
/// while they look the same; disabled and muted collide the same way.
const FROM_PRIMARY: f64 = 12.0;
const FROM_MUTED: f64 = 10.0;
const FROM_BACKGROUND: f64 = 8.0;

/// The accent, moved away from the background so a focused element outshines a primary one.
pub(super) fn focus(accent: Rgb, background: Rgb) -> Rgb {
    let accent_tone = tone(accent);
    let outward = if accent_tone >= tone(background) {
        accent_tone + FROM_PRIMARY
    } else {
        accent_tone - FROM_PRIMARY
    };

    if in_scale(outward) {
        return shifted(accent, outward);
    }

    shifted(accent, mirrored(accent_tone, outward))
}

/// The muted color, moved toward the background so a disabled element reads as switched off —
/// unless the two sit too close to fit anything between them, where it steps back the other way.
pub(super) fn disabled(muted: Rgb, background: Rgb) -> Rgb {
    let muted_tone = tone(muted);
    let background_tone = tone(background);
    let inward = if muted_tone > background_tone {
        muted_tone - FROM_MUTED
    } else {
        muted_tone + FROM_MUTED
    };

    if in_scale(inward) && (inward - background_tone).abs() >= FROM_BACKGROUND {
        return shifted(muted, inward);
    }

    shifted(muted, mirrored(muted_tone, inward))
}

/// Which way a color moves relative to a reference, along its own ramp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Way {
    Away,
    Toward,
}

/// Moves `color` `margin` tone points away from, or toward, `reference`, on the side it already
/// sits. A tie counts as the lighter side. A target that leaves 0–100 is mirrored back through the
/// origin instead of clamped, so the move still shows.
pub(super) fn pushed(color: Rgb, reference: Rgb, way: Way, margin: f64) -> Rgb {
    let origin = tone(color);
    let lighter = origin >= tone(reference);
    let target = match (way, lighter) {
        (Way::Away, true) | (Way::Toward, false) => origin + margin,
        (Way::Away, false) | (Way::Toward, true) => origin - margin,
    };
    let landed = if in_scale(target) {
        target
    } else {
        mirrored(origin, target)
    };

    shifted(color, landed)
}

fn mirrored(origin: f64, rejected: f64) -> f64 {
    origin - (rejected - origin)
}

fn in_scale(tone: f64) -> bool {
    (0.0..=100.0).contains(&tone)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(hex: &str) -> Rgb {
        Rgb::parse("k", hex).unwrap()
    }

    #[test]
    fn pushes_away_on_both_sides_of_the_reference() {
        let reference = rgb("#808080");
        let lighter = pushed(rgb("#d0d0d0"), reference, Way::Away, 6.0);
        let darker = pushed(rgb("#303030"), reference, Way::Away, 6.0);

        assert!(tone(lighter) > tone(rgb("#d0d0d0")));
        assert!(tone(darker) < tone(rgb("#303030")));
    }

    #[test]
    fn pushes_toward_on_both_sides_of_the_reference() {
        let reference = rgb("#808080");
        let lighter = pushed(rgb("#d0d0d0"), reference, Way::Toward, 8.0);
        let darker = pushed(rgb("#303030"), reference, Way::Toward, 8.0);

        assert!(tone(lighter) < tone(rgb("#d0d0d0")));
        assert!(tone(darker) > tone(rgb("#303030")));
    }

    #[test]
    fn mirrors_a_target_that_leaves_either_end_of_the_scale() {
        let white = rgb("#ffffff");
        let black = rgb("#000000");
        let from_white = pushed(white, black, Way::Away, 12.0);
        let from_black = pushed(black, white, Way::Away, 12.0);

        assert!(tone(from_white) < tone(white));
        assert!(tone(from_black) > tone(black));
    }

    #[test]
    fn treats_a_tie_with_the_reference_as_the_lighter_side() {
        let tied = rgb("#808080");
        let away = pushed(tied, tied, Way::Away, 6.0);

        assert!(tone(away) > tone(tied));
    }
}
