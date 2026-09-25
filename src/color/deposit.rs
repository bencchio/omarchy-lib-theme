//! How a role's color sits on a piece of interface, and in what situation that piece is.

use crate::theme::{Rgb, ThemeColors};

use super::Roles;
use super::content::{self, Level};
use super::ramp::{self, shifted};
use super::role::Role;
use super::tone::{self, separation};
use super::variants::{self, Way};

/// How far a hover steps away from the canvas, in tone points.
const HOVER: f64 = 6.0;
/// How far a press steps toward the canvas, in tone points.
const PRESSED: f64 = 8.0;
/// How far focus steps away from the canvas — the same margin `focus` already uses.
const FOCUSED: f64 = 12.0;
/// How far a disabled piece steps toward the canvas — the same margin `disabled` already uses.
const DISABLED: f64 = 10.0;
/// How far a track's indicator must stand from the track — the disabled-content floor.
pub(super) const TRACK_APART: f64 = 18.0;
/// How far a state has to move a color before it reads as a different situation.
const STATE_APART: f64 = 4.0;

/// The way a color is deposited on a piece. None of these names a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Deposit {
    /// A filled piece, such as a primary button.
    Fill,
    /// A line around a piece, such as a ghost button.
    Outline,
    /// A line under a piece, such as an active tab.
    Underline,
    /// Marked text, such as a selection inside a paragraph.
    Highlight,
    /// A track and the indicator that travels it, such as a progress bar.
    Track,
}

impl Deposit {
    pub const ALL: [Self; 5] = [
        Self::Fill,
        Self::Outline,
        Self::Underline,
        Self::Highlight,
        Self::Track,
    ];
}

/// The situation a piece is in. Composes with [`Deposit`]; neither replaces the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    /// The piece at rest.
    Rest,
    /// The pointer is over the piece.
    Hover,
    /// The piece is being pressed.
    Pressed,
    /// The piece cannot be interacted with.
    Disabled,
    /// The piece holds keyboard focus.
    Focused,
}

impl State {
    pub const ALL: [Self; 5] = [
        Self::Rest,
        Self::Hover,
        Self::Pressed,
        Self::Disabled,
        Self::Focused,
    ];
}

/// The two colors a deposit needs to draw.
///
/// What each field holds depends on the deposit:
/// - [`Deposit::Fill`] and [`Deposit::Highlight`]: `ground` is the fill, `mark` is the content that
///   reads on it.
/// - [`Deposit::Outline`] and [`Deposit::Underline`]: `ground` is the line, `mark` is the content
///   that already reads on the canvas — the piece sits on the canvas, so the role's own content
///   would be read against the wrong ground.
/// - [`Deposit::Track`]: `ground` is the track, `mark` is the indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Styled {
    pub ground: Rgb,
    pub mark: Rgb,
}

pub(crate) fn resolve(
    roles: &Roles,
    colors: &ThemeColors,
    role: Role,
    deposit: Deposit,
    state: State,
) -> Styled {
    let pushed = apply_state(roles.color(role), roles.color(Role::Background), state);

    match deposit {
        Deposit::Fill | Deposit::Highlight => Styled {
            ground: pushed,
            mark: mark_on(pushed, colors, state),
        },
        Deposit::Outline | Deposit::Underline => Styled {
            ground: pushed,
            mark: roles.content(Role::Background, content_level(state)),
        },
        Deposit::Track => {
            let role_color = roles.color(role);
            let ground = track_ground(roles, state);
            let rest_mark = track_indicator(
                role_color,
                role_color,
                track_ground(roles, State::Rest),
                false,
            );
            let mark = if state == State::Rest {
                rest_mark
            } else {
                let mut marked = track_indicator(role_color, pushed, ground, true);
                if state == State::Disabled {
                    marked = super::desaturate(marked);
                }
                if separation(marked, rest_mark) < STATE_APART {
                    let delta = (tone::tone(pushed) - tone::tone(role_color))
                        .abs()
                        .max(STATE_APART);
                    marked = step_past(rest_mark, ground, delta);
                    if state == State::Disabled {
                        marked = super::desaturate(marked);
                    }
                }
                marked
            };
            Styled { ground, mark }
        }
    }
}

fn apply_state(color: Rgb, background: Rgb, state: State) -> Rgb {
    let pushed = match state {
        State::Rest => return color,
        State::Hover => variants::pushed(color, background, Way::Away, HOVER),
        State::Pressed => variants::pushed(color, background, Way::Toward, PRESSED),
        State::Disabled => variants::pushed(color, background, Way::Toward, DISABLED),
        State::Focused => variants::pushed(color, background, Way::Away, FOCUSED),
    };

    if state == State::Disabled {
        super::desaturate(pushed)
    } else {
        pushed
    }
}

fn content_level(state: State) -> Level {
    if state == State::Disabled {
        Level::Disabled
    } else {
        Level::Primary
    }
}

fn mark_on(ground: Rgb, colors: &ThemeColors, state: State) -> Rgb {
    if state == State::Disabled {
        content::levels(ground, colors).at(Level::Disabled)
    } else {
        content::on(ground, colors)
    }
}

fn track_ground(roles: &Roles, state: State) -> Rgb {
    let recessed = roles.color(Role::Recessed);
    if state == State::Disabled {
        super::desaturate(recessed)
    } else {
        recessed
    }
}

fn kept_apart(indicator: Rgb, track: Rgb) -> Rgb {
    if separation(indicator, track) >= TRACK_APART {
        return indicator;
    }

    let track_tone = tone::tone(track);
    let lighter = tone::tone(indicator) >= track_tone;
    let placed = place_apart(indicator, track, track_tone, lighter);
    if separation(placed, track) >= TRACK_APART {
        return placed;
    }

    place_apart(indicator, track, track_tone, !lighter)
}

/// The ramp lands near the tone it is asked for, not on it, so a single step to the edge can still
/// fall short of [`TRACK_APART`]. Walk outward until the gap holds, or the scale runs out.
fn place_apart(indicator: Rgb, track: Rgb, track_tone: f64, lighter: bool) -> Rgb {
    let mut target = ramp::edge(track_tone, TRACK_APART, lighter);
    let mut placed = shifted(indicator, target);
    let step = if lighter { 1.0 } else { -1.0 };

    while separation(placed, track) < TRACK_APART && (0.0..=100.0).contains(&target) {
        target += step;
        placed = shifted(indicator, target);
    }
    placed
}

/// The indicator is the role color after the state push, held at least [`TRACK_APART`] from the
/// track. When that floor lands the state on the same tone as rest, the push is reapplied past the
/// floor, away from the track — otherwise a color that already sits on the track would look the
/// same at rest and on hover.
fn track_indicator(rest_color: Rgb, pushed: Rgb, track: Rgb, changed: bool) -> Rgb {
    let rested = kept_apart(rest_color, track);
    if !changed {
        return rested;
    }

    let marked = kept_apart(pushed, track);
    if separation(marked, rested) >= STATE_APART {
        return marked;
    }

    let delta = (tone::tone(pushed) - tone::tone(rest_color))
        .abs()
        .max(STATE_APART);

    step_past(rested, track, delta)
}

/// Moves `rested` by `delta` on the side that still has room. Away from the track first; the other
/// way when that side of the scale is already used up, so a state still reads.
fn step_past(rested: Rgb, track: Rgb, delta: f64) -> Rgb {
    let origin = tone::tone(rested);
    let away_lighter = origin >= tone::tone(track);
    let mut extra = 0.0;

    loop {
        let span = delta + extra;
        let away = if away_lighter {
            origin + span
        } else {
            origin - span
        };
        let stepped = shifted(rested, away);
        if separation(stepped, rested) >= STATE_APART && separation(stepped, track) >= TRACK_APART {
            return stepped;
        }

        let back = if away_lighter {
            origin - span
        } else {
            origin + span
        };
        let stepped = shifted(rested, back);
        if separation(stepped, rested) >= STATE_APART && separation(stepped, track) >= TRACK_APART {
            return stepped;
        }

        extra += 1.0;
        if extra > 20.0 {
            return kept_apart(stepped, track);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Palette, PaletteSeed};
    use crate::modes::Treatment;
    use std::path::{Path, PathBuf};

    const READABLE: f64 = 40.0;
    const DISABLED_FLOOR: f64 = 15.0;
    const STATE_MOVE: f64 = STATE_APART;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    fn palette(name: &str) -> Palette {
        PaletteSeed::from_path(&fixture(name)).unwrap().extend()
    }

    fn meets_floors(palette: &Palette) {
        let canvas = palette.color(Role::Background);

        for role in Role::ALL {
            for deposit in Deposit::ALL {
                let rest = palette.deposit(role, deposit, State::Rest);

                for state in State::ALL {
                    let styled = palette.deposit(role, deposit, state);

                    match deposit {
                        Deposit::Fill | Deposit::Highlight => {
                            let floor = if state == State::Disabled {
                                DISABLED_FLOOR
                            } else {
                                READABLE
                            };
                            assert!(
                                separation(styled.mark, styled.ground) >= floor,
                                "{role:?} {deposit:?} {state:?}: mark/ground {}",
                                separation(styled.mark, styled.ground)
                            );
                        }
                        Deposit::Outline | Deposit::Underline => {
                            let floor = if state == State::Disabled {
                                DISABLED_FLOOR
                            } else {
                                READABLE
                            };
                            assert!(
                                separation(styled.mark, canvas) >= floor,
                                "{role:?} {deposit:?} {state:?}: mark/canvas {}",
                                separation(styled.mark, canvas)
                            );
                        }
                        Deposit::Track => {
                            assert!(
                                separation(styled.mark, styled.ground) >= TRACK_APART,
                                "{role:?} {state:?}: track gap {}",
                                separation(styled.mark, styled.ground)
                            );
                        }
                    }

                    if state == State::Rest {
                        continue;
                    }

                    let (moved, rested) = if deposit == Deposit::Track {
                        (styled.mark, rest.mark)
                    } else {
                        (styled.ground, rest.ground)
                    };
                    let shifted_enough = separation(moved, rested) >= STATE_MOVE;
                    let lost_chroma = state == State::Disabled
                        && tone::chroma(moved) < tone::chroma(rested) / 4.0;
                    assert!(
                        shifted_enough || lost_chroma,
                        "{role:?} {deposit:?} {state:?}: moved {} chroma {} -> {}",
                        separation(moved, rested),
                        tone::chroma(rested),
                        tone::chroma(moved)
                    );
                }

                for state in State::ALL {
                    assert_eq!(
                        palette.deposit(role, Deposit::Highlight, state),
                        palette.deposit(role, Deposit::Fill, state),
                        "{role:?} {state:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn holds_its_floors_on_the_fixture_themes() {
        for name in [
            "everforest.toml",
            "flexoki-light.toml",
            "flat-surface.toml",
            "light-on-white.toml",
            "foreground-variant-reads.toml",
        ] {
            meets_floors(&palette(name));
        }
    }

    #[test]
    fn holds_its_floors_under_every_treatment() {
        let palette = palette("everforest.toml");

        for treatment in Treatment::ALL {
            meets_floors(&palette.represented(treatment));
        }
    }
}
