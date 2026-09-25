//! The palette contract: basic in, extended and readable out.

use crate::color::{Deposit, Level, Ramp, Role, Roles, State, Styled, tone_of};
use crate::modes::{Treatment, represent};
use crate::theme::{
    Family, Mode, Rgb, Shades, Theme, ThemeColors, ThemeError, brighter, darker, read_theme,
};

use crate::color;

/// The palette an application or the theme reader starts from: a few core colors.
///
/// `background`, `foreground` and `accent` are required; everything else is optional and, when
/// left unstated, the library derives a readable value for it as soon as it's built. This is the
/// one loading surface: both the active Omarchy theme and an application-built palette enter
/// through these constructors, and neither loses a key the other keeps — [`from_theme`](Self::from_theme)
/// carries the whole [`Theme`] across, unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteSeed {
    theme: Theme,
}

impl PaletteSeed {
    /// The basic palette of the system's active Omarchy theme.
    pub fn from_active_theme() -> Result<Self, ThemeError> {
        Self::from_path(&crate::theme::default_theme_path())
    }

    /// The basic palette of an Omarchy-style theme read from `path`.
    pub fn from_path(path: &std::path::Path) -> Result<Self, ThemeError> {
        let theme = read_theme(path)?;
        Ok(Self::from_theme(&theme))
    }

    /// The basic palette of a [`Theme`] already read, such as one a [`ThemeWatcher`](crate::theme::ThemeWatcher)
    /// holds. Lossless: every key `theme` carries survives into [`extend`](Self::extend) untouched,
    /// which is what lets this be the one loading surface every other constructor here builds on.
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            theme: theme.clone(),
        }
    }

    /// A basic palette built by the application itself, from the three colors it owns.
    pub fn new(background: Rgb, foreground: Rgb, accent: Rgb) -> Self {
        Self {
            theme: completed(background, foreground, accent),
        }
    }

    /// The ground behind selected content. Defaults to the accent.
    pub fn with_selection(mut self, color: Rgb) -> Self {
        self.theme.colors.selection = color;
        self
    }

    /// De-emphasized content. Defaults to the foreground.
    pub fn with_muted(mut self, color: Rgb) -> Self {
        self.theme.colors.muted = color;
        self
    }

    /// The color that signals something went wrong. Defaults to a documented red. Its dark and
    /// bright shades are derived from `color`, the same way a theme's own declared shades are.
    pub fn with_error(mut self, color: Rgb) -> Self {
        self.theme.colors.red = color;
        self.theme.colors.dark_red = darker(color);
        self.theme.colors.bright_red = brighter(color);
        self
    }

    /// The raised surface. Defaults to a lighter step of the background's own ramp.
    pub fn with_surface_light(mut self, color: Rgb) -> Self {
        self.theme.colors.lighter_background = color;
        self
    }

    /// The recessed surface. Defaults to a darker step of the background's own ramp.
    pub fn with_surface_dark(mut self, color: Rgb) -> Self {
        self.theme.colors.dark_background = color;
        self
    }

    /// The mode the palette reads as, instead of the one inferred from the background's tone.
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.theme.mode = mode;
        self
    }

    /// The extended legible palette this basic one reads as: roles, content and tonal ramps.
    pub fn extend(&self) -> Palette {
        resolve(self.theme.clone(), Treatment::Original)
    }
}

/// The error color a basic palette gets when the application does not state one. Red, distinct
/// from the accent, so a failing element reads as such no matter the theme.
const DEFAULT_ERROR: Rgb = Rgb {
    red: 229,
    green: 72,
    blue: 77,
};

/// The full [`Theme`] an app-built seed reads as: every key [`PaletteSeed::new`] leaves unstated
/// gets a readable default here, once, at construction — the same policy [`PaletteSeed`]'s builders
/// then overwrite one key at a time.
fn completed(background: Rgb, foreground: Rgb, accent: Rgb) -> Theme {
    let background_tone = tone_of(background);
    let mode = if background_tone < 50.0 {
        Mode::Dark
    } else {
        Mode::Light
    };

    Theme {
        mode,
        colors: ThemeColors {
            accent,
            selection: accent,
            muted: foreground,

            background,
            dark_background: Ramp::of(background).tone((background_tone - 15.0) as u8),
            darker_background: Ramp::of(background).tone((background_tone - 30.0) as u8),
            lighter_background: Ramp::of(background).tone((background_tone + 15.0) as u8),

            foreground,
            dark_foreground: foreground,
            light_foreground: foreground,
            bright_foreground: foreground,

            red: DEFAULT_ERROR,
            dark_red: darker(DEFAULT_ERROR),
            bright_red: brighter(DEFAULT_ERROR),

            yellow: foreground,
            dark_yellow: foreground,
            bright_yellow: foreground,

            green: foreground,
            dark_green: foreground,
            bright_green: foreground,

            cyan: foreground,
            dark_cyan: foreground,
            bright_cyan: foreground,

            blue: foreground,
            dark_blue: foreground,
            bright_blue: foreground,

            magenta: foreground,
            dark_magenta: foreground,
            bright_magenta: foreground,
        },
    }
}

/// Resolves `source` through `treatment`: the only place [`represent`] and [`Roles::resolve`] are
/// called from. Every [`Palette`] in the crate is built here, so Rust and the C bridge — which
/// builds its snapshot by marshalling a `Palette` — read the same values by construction.
fn resolve(source: Theme, treatment: Treatment) -> Palette {
    let resolved = represent(&source, treatment);
    let roles = Roles::resolve(&resolved);
    Palette {
        source,
        treatment,
        resolved,
        roles,
    }
}

/// The extended legible palette: what an application actually draws with.
///
/// Resolved once from a [`PaletteSeed`]; every color and ramp here already guarantees its content
/// stays readable on its ground. This is the v1 public contract of the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    source: Theme,
    treatment: Treatment,
    resolved: Theme,
    roles: Roles,
}

impl Palette {
    /// The concrete color of a semantic role, such as the accent or the canvas.
    pub fn color(&self, role: Role) -> Rgb {
        self.roles.color(role)
    }

    /// The main content that stays readable on `role`.
    pub fn on(&self, role: Role) -> Rgb {
        self.roles.on(role)
    }

    /// The content of a given weight that reads on `role`.
    pub fn content(&self, role: Role, level: Level) -> Rgb {
        self.roles.content(role, level)
    }

    /// How `role` sits on a piece in `state`: the two colors that deposit needs to draw.
    ///
    /// Works under any treatment, because it reads the palette already resolved — not the source
    /// theme.
    pub fn deposit(&self, role: Role, deposit: Deposit, state: State) -> Styled {
        crate::color::deposit::resolve(&self.roles, &self.resolved.colors, role, deposit, state)
    }

    /// The tonal ramp a role scales along, from black to white at its hue.
    pub fn ramp(&self, role: Role) -> Ramp {
        self.roles.ramp(role)
    }

    /// Whether the palette is built on a dark ground.
    pub fn dark(&self) -> bool {
        self.resolved.mode.is_dark()
    }

    /// Content that stays readable on `color`, whether or not it plays a role here — the same
    /// guarantee the roles carry, lifted onto any color the application wants to draw with.
    pub fn on_color(&self, color: Rgb) -> Rgb {
        color::on(color, &self.resolved.colors)
    }

    /// The same palette, taken through `treatment` instead — always re-resolved from the original
    /// source, never stacked on top of the current one, so `represented(a).represented(b)` is the
    /// same as `represented(b)`.
    pub(crate) fn represented(&self, treatment: Treatment) -> Palette {
        resolve(self.source.clone(), treatment)
    }

    /// One hue family's color and shades, as this palette resolved it.
    pub(crate) fn family(&self, family: Family) -> Shades {
        self.resolved.colors.family(family)
    }
}
