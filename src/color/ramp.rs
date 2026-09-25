//! Tonal ramps: one base color taken across the whole lightness range at constant hue.

use material_colors::palette::TonalPalette;

use crate::theme::Rgb;

use super::tone::{from_argb, to_hct};

/// The tones a base color takes from 0 (black) to 100 (white), keeping its hue and chroma.
#[derive(Debug, Clone)]
pub struct Ramp {
    palette: TonalPalette,
}

impl Ramp {
    /// The ramp any color takes, whether or not it plays a role in the theme.
    pub fn of(base: Rgb) -> Self {
        Self {
            palette: TonalPalette::from_hct(to_hct(base)),
        }
    }

    /// Values above 100 are clamped, so callers cannot ask for a tone off the scale.
    pub fn tone(&self, tone: u8) -> Rgb {
        from_argb(self.palette.tone(i32::from(tone.min(100))))
    }
}

/// The same color taken to another point of its own ramp, keeping its hue.
pub(super) fn shifted(color: Rgb, tone: f64) -> Rgb {
    let step = tone.clamp(0.0, 100.0).round() as u8;
    Ramp::of(color).tone(step)
}

/// A ramp lands near the tone it is asked for, not exactly on it, so a tone forced to clear a
/// minimum separation overshoots it by one step instead of landing right on the edge and falling
/// back under it on rounding.
const OVERSHOOT: f64 = 1.0;

/// The tone that clears `minimum` distance from `reference`, on the lighter or the darker side.
pub(super) fn edge(reference: f64, minimum: f64, lighter: bool) -> f64 {
    if lighter {
        (reference + minimum).ceil() + OVERSHOOT
    } else {
        (reference - minimum).floor() - OVERSHOOT
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::tone::tone as tone_of;

    fn ramp() -> Ramp {
        Ramp::of(Rgb::parse("k", "#7fbbb3").unwrap())
    }

    #[test]
    fn runs_from_black_to_white() {
        assert_eq!(ramp().tone(0), Rgb::parse("k", "#000000").unwrap());
        assert_eq!(ramp().tone(100), Rgb::parse("k", "#ffffff").unwrap());
    }

    #[test]
    fn grows_lighter_with_every_step() {
        let ramp = ramp();
        let mut previous = -1.0;

        for step in (0..=100).step_by(10) {
            let current = tone_of(ramp.tone(step));
            assert!(
                current > previous,
                "tone {step} did not rise above the previous step"
            );
            previous = current;
        }
    }

    #[test]
    fn clamps_beyond_the_end_of_the_scale() {
        assert_eq!(ramp().tone(255), ramp().tone(100));
    }

    #[test]
    fn keeps_the_hue_of_its_base() {
        let base = Rgb::parse("k", "#7fbbb3").unwrap();
        let base_hue = super::to_hct(base).get_hue();

        for step in [30, 50, 70] {
            let hue = super::to_hct(ramp().tone(step)).get_hue();
            assert!(
                (hue - base_hue).abs() < 5.0,
                "tone {step} drifted from hue {base_hue:.1} to {hue:.1}"
            );
        }
    }
}
