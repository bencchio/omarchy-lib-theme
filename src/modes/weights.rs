//! Which tone each color takes when a treatment leaves them one hue and nothing else to tell them apart.

use crate::color::{Ramp, tone_of};
use crate::theme::Rgb;

/// How far two tones stand apart to still read as two.
pub(super) const APART: f64 = 5.0;

/// The band the colors keep to: dark enough to carry on a light page, light enough not to be taken
/// for the text, which the treatment pins near the bottom of the scale.
const DARKEST: f64 = 15.0;
const LIGHTEST: f64 = 65.0;

/// The tones already spoken for.
///
/// Hue is what told a theme's colors apart and a one-hue page has none, so two of them can land on
/// the same tone — a theme whose red and blue read alike would show an error the way it shows a
/// primary. Each color keeps the tone its own weight gives it, and only one landing where another
/// already sits is moved, as little as moving takes.
pub(super) struct Weights {
    taken: Vec<f64>,
    paint: fn(Rgb, f64) -> Rgb,
}

impl Weights {
    /// `paint` turns a color and the tone it was given into what the page delivers.
    pub(super) fn new(paint: fn(Rgb, f64) -> Rgb) -> Self {
        Self {
            taken: Vec::new(),
            paint,
        }
    }

    pub(super) fn place(&mut self, color: Rgb) -> Rgb {
        let free = self.free(tone_of(color));
        self.taken.push(free);

        (self.paint)(color, free)
    }

    /// A color and the shades that belong to it: the shades follow it, keeping the distance the
    /// theme gave them, so a family moves without coming apart.
    pub(super) fn family(&mut self, original: Rgb, dark: Rgb, bright: Rgb) -> (Rgb, Rgb, Rgb) {
        let placed = self.place(original);
        let shift = tone_of(placed) - tone_of(original);

        (
            placed,
            (self.paint)(dark, tone_of(dark) + shift),
            (self.paint)(bright, tone_of(bright) + shift),
        )
    }

    /// The tone nearest `tone` that no color has taken, searched outward a step at a time so one
    /// already sitting clear does not move at all.
    fn free(&self, tone: f64) -> f64 {
        let wanted = tone.clamp(DARKEST, LIGHTEST);
        let steps = ((LIGHTEST - DARKEST) / APART).ceil() as i32;

        (0..=steps)
            .flat_map(|step| {
                let offset = f64::from(step) * APART;
                [wanted - offset, wanted + offset]
            })
            .filter(|candidate| (DARKEST..=LIGHTEST).contains(candidate))
            .find(|candidate| self.is_free(*candidate))
            .unwrap_or(wanted)
    }

    fn is_free(&self, tone: f64) -> bool {
        !self.taken.iter().any(|taken| (taken - tone).abs() < APART)
    }
}

/// `color` moved along its own ramp to `tone`, keeping its hue.
pub(super) fn at(color: Rgb, tone: f64) -> Rgb {
    Ramp::of(color).tone(tone.clamp(0.0, 100.0).round() as u8)
}
