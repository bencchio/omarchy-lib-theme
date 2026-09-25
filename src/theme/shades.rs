//! The darker and lighter shades of a color, for the themes that declare a color without them.

use material_colors::color::Argb;
use material_colors::hct::Hct;
use material_colors::palette::TonalPalette;

use super::rgb::Rgb;

/// How far a derived shade stands from the color it belongs to.
///
/// Measured over the Omarchy catalog rather than chosen: of the base-and-bright pairs a theme
/// distinguishes at all, this is the gap three quarters of them stay within. The median is half of
/// it, which is too little to tell apart on screen, and the catalog gives no dark pairs to measure
/// — so the dark shade drops by what the light one climbs, which is the one thing not invented.
const STEP: f64 = 14.0;

/// `color` lifted along its own tonal ramp, keeping its hue and chroma. A color already at the top
/// of the scale stays where it is rather than washing out.
pub(crate) fn brighter(color: Rgb) -> Rgb {
    shifted(color, STEP)
}

/// `color` taken down its own tonal ramp by the same step, keeping its hue and chroma. A color
/// already at the bottom stays there rather than turning black.
pub(crate) fn darker(color: Rgb) -> Rgb {
    shifted(color, -STEP)
}

fn shifted(color: Rgb, step: f64) -> Rgb {
    let hct = Hct::new(Argb::new(u8::MAX, color.red, color.green, color.blue));
    let moved = (hct.get_tone() + step).clamp(0.0, 100.0).round() as i32;
    let argb = TonalPalette::from_hct(hct).tone(moved);

    Rgb {
        red: argb.red,
        green: argb.green,
        blue: argb.blue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifts_a_color_off_its_base() {
        let base = Rgb::parse("red", "#c34043").unwrap();
        let bright = brighter(base);

        assert_ne!(bright, base);
    }

    #[test]
    fn drops_a_color_below_its_base() {
        let base = Rgb::parse("red", "#c34043").unwrap();

        assert_ne!(darker(base), base);
        assert_ne!(darker(base), brighter(base));
    }

    #[test]
    fn leaves_a_color_at_the_top_of_the_scale_alone() {
        let white = Rgb::parse("white", "#ffffff").unwrap();

        assert_eq!(brighter(white), white);
    }

    #[test]
    fn leaves_a_color_at_the_bottom_of_the_scale_alone() {
        let black = Rgb::parse("black", "#000000").unwrap();

        assert_eq!(darker(black), black);
    }
}
