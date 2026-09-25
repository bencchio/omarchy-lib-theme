//! A raw theme color, as `colors.toml` declares it, with no semantic meaning attached.

use std::fmt;

use super::error::ThemeError;

/// An 8-bit-per-channel color read straight from the theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    /// Accepts `#rrggbb` in either case, the only form Omarchy themes use for the keys that
    /// belong to the color contract.
    pub fn parse(key: &'static str, value: &str) -> Result<Self, ThemeError> {
        let invalid = || ThemeError::InvalidColor(key, value.to_owned());

        let digits = value.strip_prefix('#').ok_or_else(invalid)?;
        if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(invalid());
        }

        let channel = |start: usize| u8::from_str_radix(&digits[start..start + 2], 16);
        match (channel(0), channel(2), channel(4)) {
            (Ok(red), Ok(green), Ok(blue)) => Ok(Self { red, green, blue }),
            _ => Err(invalid()),
        }
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "#{:02x}{:02x}{:02x}",
            self.red, self.green, self.blue
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_lowercase_color() {
        let color = Rgb::parse("background", "#2d353b").unwrap();
        assert_eq!(
            color,
            Rgb {
                red: 0x2d,
                green: 0x35,
                blue: 0x3b
            }
        );
    }

    #[test]
    fn reads_an_uppercase_color() {
        assert_eq!(
            Rgb::parse("accent", "#B91C1C").unwrap(),
            Rgb::parse("accent", "#b91c1c").unwrap()
        );
    }

    #[test]
    fn reads_the_extremes() {
        assert_eq!(
            Rgb::parse("k", "#000000").unwrap(),
            Rgb {
                red: 0,
                green: 0,
                blue: 0
            }
        );
        assert_eq!(
            Rgb::parse("k", "#ffffff").unwrap(),
            Rgb {
                red: 255,
                green: 255,
                blue: 255
            }
        );
    }

    #[test]
    fn rejects_anything_that_is_not_rrggbb() {
        for value in [
            "2d353b",
            "#2d353",
            "#2d353bb",
            "#gggggg",
            "",
            "#",
            "rgb(1e1e1e)",
        ] {
            let error = Rgb::parse("background", value).unwrap_err();
            assert!(
                matches!(&error, ThemeError::InvalidColor("background", got) if got == value),
                "should have rejected `{value}`"
            );
        }
    }

    #[test]
    fn renders_back_the_way_the_theme_declares_it() {
        assert_eq!(Rgb::parse("k", "#2D353B").unwrap().to_string(), "#2d353b");
    }
}
