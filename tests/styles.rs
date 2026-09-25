//! The style guarantees, held on every theme Omarchy installs and under every treatment.

use std::fs;
use std::path::{Path, PathBuf};

use omarchy_lib_theme::{
    Deposit, Palette, PaletteSeed, Role, State, Treatment, chroma_of, read_theme, represent,
    separation,
};

/// The roles whose lines have to be seen. `Border` and `Muted` are quiet by design, and the planes
/// and the canvas are what lines are drawn on.
const LINE_ROLES: [Role; 3] = [Role::Primary, Role::Error, Role::Focus];

const THEMES_DIR: &str = "/usr/share/omarchy/themes";

/// How far a piece has to stand from what it sits on to be seen.
const SEEN: f64 = 40.0;
/// How far a line has to stand from the canvas to be seen: strokes need less than text does.
const LINE_SEEN: f64 = 15.0;
/// How far a disabled piece can fall and still be read.
const DISABLED_READABLE: f64 = 15.0;
/// How far a state has to move a color before it reads as a different situation.
const STATE_APART: f64 = 4.0;
/// How far a track's indicator has to stand from the track.
const TRACK_APART: f64 = 18.0;

/// The `colors.toml` of every theme directly under `directory`, sorted; empty when it does not exist.
fn installed_themes(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut themes: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path().join("colors.toml"))
        .filter(|path| path.is_file())
        .collect();
    themes.sort();
    themes
}

fn theme_name(path: &Path) -> String {
    path.parent()
        .and_then(Path::file_name)
        .map_or_else(|| "?".into(), |name| name.to_string_lossy().into_owned())
}

/// Every broken guarantee on `palette`, each named by where it broke.
fn broken(palette: &Palette, label: &str) -> Vec<String> {
    let canvas = palette.color(Role::Background);
    let mut problems = Vec::new();

    for role in Role::ALL {
        for deposit in Deposit::ALL {
            let rest = palette.deposit(role, deposit, State::Rest);

            for state in State::ALL {
                let styled = palette.deposit(role, deposit, state);
                let at = format!("{label} · {role:?} {deposit:?} {state:?}");

                // What the mark is read against: its own ground, or the canvas the piece sits on.
                let read_against = match deposit {
                    Deposit::Fill | Deposit::Highlight | Deposit::Track => styled.ground,
                    Deposit::Outline | Deposit::Underline => canvas,
                };
                let floor = if state == State::Disabled {
                    DISABLED_READABLE
                } else {
                    SEEN
                };

                if deposit != Deposit::Track {
                    let read = separation(styled.mark, read_against);
                    if read < floor {
                        problems.push(format!("{at}: mark reads {read:.1}, needs {floor}"));
                    }
                }

                match deposit {
                    Deposit::Outline | Deposit::Underline
                        if LINE_ROLES.contains(&role) && state != State::Disabled =>
                    {
                        let seen = separation(styled.ground, canvas);
                        if seen < LINE_SEEN {
                            problems.push(format!(
                                "{at}: line sits {seen:.1} from the background, needs {LINE_SEEN}"
                            ));
                        }
                    }
                    Deposit::Track => {
                        let gap = separation(styled.mark, styled.ground);
                        if gap < TRACK_APART {
                            problems.push(format!(
                                "{at}: indicator sits {gap:.1} from the track, needs {TRACK_APART}"
                            ));
                        }
                    }
                    _ => {}
                }

                if state == State::Rest {
                    continue;
                }

                let (moved, rested) = if deposit == Deposit::Track {
                    (styled.mark, rest.mark)
                } else {
                    (styled.ground, rest.ground)
                };
                let shifted = separation(moved, rested) >= STATE_APART;
                let dimmed = state == State::Disabled && chroma_of(moved) < chroma_of(rested) / 4.0;

                if !shifted && !dimmed {
                    problems.push(format!(
                        "{at}: looks the same as at rest ({:.1} apart)",
                        separation(moved, rested)
                    ));
                }
            }
        }
    }

    problems
}

fn check(themes: &[PathBuf]) -> Vec<String> {
    let mut problems = Vec::new();

    for path in themes {
        let name = theme_name(path);
        let theme = read_theme(path).unwrap_or_else(|error| panic!("{name}: {error}"));

        for treatment in Treatment::ALL {
            let palette = PaletteSeed::from_theme(&represent(&theme, treatment)).extend();
            problems.extend(broken(&palette, &format!("{name} · {treatment:?}")));
        }
    }

    problems
}

#[test]
fn every_installed_theme_holds_the_style_guarantees() {
    let themes = installed_themes(Path::new(THEMES_DIR));

    if themes.is_empty() {
        eprintln!("skipped: no installed themes under {THEMES_DIR}");
        return;
    }

    let problems = check(&themes);
    println!(
        "{} themes × {} treatments",
        themes.len(),
        Treatment::ALL.len()
    );

    assert!(
        problems.is_empty(),
        "{} broken:\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn a_missing_themes_directory_finds_nothing() {
    assert!(installed_themes(Path::new("/nonexistent/omarchy/themes")).is_empty());
}
