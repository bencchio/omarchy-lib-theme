//! Bridge between the theme's raw colors and the perceptual tone math the roles are resolved with.

use material_colors::color::Argb;
use material_colors::hct::Hct;

use crate::theme::Rgb;

pub(crate) fn to_hct(color: Rgb) -> Hct {
    Hct::new(Argb::new(u8::MAX, color.red, color.green, color.blue))
}

pub(crate) fn from_argb(argb: Argb) -> Rgb {
    Rgb {
        red: argb.red,
        green: argb.green,
        blue: argb.blue,
    }
}

/// Perceptual lightness, 0 (black) to 100 (white).
pub(crate) fn tone(color: Rgb) -> f64 {
    to_hct(color).get_tone()
}

pub(crate) fn separation(one: Rgb, other: Rgb) -> f64 {
    (tone(one) - tone(other)).abs()
}

/// How saturated a color reads, 0 (gray) upward. A ground below this has no hue worth keeping, so
/// nothing forces its content into a color family.
pub(crate) fn chroma(color: Rgb) -> f64 {
    to_hct(color).get_chroma()
}

/// How far two colors stand apart in hue, 0 to 180.
pub(crate) fn hue_gap(one: Rgb, other: Rgb) -> f64 {
    let gap = (to_hct(one).get_hue() - to_hct(other).get_hue()).abs();
    if gap > 180.0 { 360.0 - gap } else { gap }
}

/// The same color at zero chroma: its tone unchanged, its hue meaningless once chroma is gone.
pub(crate) fn desaturate(color: Rgb) -> Rgb {
    let mut hct = to_hct(color);
    hct.set_chroma(0.0);
    from_argb(hct.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color(hex: &str) -> Rgb {
        Rgb::parse("k", hex).unwrap()
    }

    #[test]
    fn places_black_and_white_at_the_ends_of_the_scale() {
        assert!(tone(color("#000000")) <= 0.5);
        assert!(tone(color("#ffffff")) >= 99.5);
    }

    #[test]
    fn survives_a_round_trip_through_the_color_space() {
        let original = color("#7fbbb3");
        let restored = from_argb(to_hct(original).into());

        assert_eq!(original, restored);
    }

    #[test]
    fn measures_separation_regardless_of_order() {
        let dark = color("#2d353b");
        let light = color("#d3c6aa");

        assert_eq!(separation(dark, light), separation(light, dark));
        assert!(separation(dark, light) > 40.0);
    }

    #[test]
    fn desaturating_drops_chroma_without_moving_tone() {
        let original = color("#7fbbb3");
        let flat = desaturate(original);

        // The round trip through 8-bit sRGB leaves a residue of a few units; what matters is that
        // it collapses from a real hue, not that it lands on exactly zero.
        assert!(
            chroma(flat) < chroma(original) / 4.0,
            "barely desaturated: {} from {}",
            chroma(flat),
            chroma(original)
        );
        assert!((tone(flat) - tone(original)).abs() < 1.0);
    }
}
