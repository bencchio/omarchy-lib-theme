//! Names what is still incoherent in each theme:
//! `cargo run --example audit-themes -- /usr/share/omarchy/themes/*/colors.toml`

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use omarchy_lib_theme::{
    Level, Rgb, Role, Roles, Theme, Treatment, chroma_of, hue_gap, read_theme, represent,
    separation, tone_of,
};

/// A card has to be seen without looking for it; every other plane only has to read as its own.
const CARD_FROM_BACKGROUND: f64 = 6.0;
const PLANE_FROM_PLANE: f64 = 4.0;
const ROLE_FROM_ROLE: f64 = 8.0;
const CONTENT_FROM_GROUND: f64 = 40.0;
const DISABLED_FROM_GROUND: f64 = 15.0;
/// Below this a ground has no hue worth keeping content in family with.
const CHROMATIC_GROUND: f64 = 12.0;
/// How far content can drift from its ground's hue before it reads as an unrelated color.
const MAX_HUE_DRIFT: f64 = 30.0;

fn main() -> ExitCode {
    let mut themes = 0;
    let mut problems = 0;

    for argument in std::env::args().skip(1) {
        let path = PathBuf::from(&argument);
        let name = theme_name(&path, &argument);

        let theme = match read_theme(&path) {
            Ok(theme) => theme,
            Err(error) => {
                println!("FAIL {name:<18} {error}");
                problems += 1;
                continue;
            }
        };

        themes += 1;

        for treatment in Treatment::ALL {
            let transformed = represent(&theme, treatment);
            let found = audit(&transformed);
            let mode = if transformed.mode.is_dark() {
                "dark"
            } else {
                "light"
            };
            let label = format!("{name} · {treatment:?}");

            if found.is_empty() {
                println!("ok   {label:<32} {mode}");
            } else {
                println!("FAIL {label:<32} {mode}");
                for problem in &found {
                    println!("       {problem}");
                }
                problems += found.len();
            }
        }
    }

    println!("-----");
    println!("{themes} themes · {problems} problems");

    if problems == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn audit(theme: &Theme) -> Vec<String> {
    let roles = Roles::resolve(theme);
    let mut problems = Vec::new();

    problems.extend(planes(&roles));
    problems.extend(collisions(&roles));
    problems.extend(content(&roles, theme));
    problems
}

fn planes(roles: &Roles) -> Vec<String> {
    let background = roles.color(Role::Background);
    let card = roles.color(Role::Elevated);
    let well = roles.color(Role::Recessed);
    let mut problems = Vec::new();

    let off_background = separation(card, background);
    if off_background < CARD_FROM_BACKGROUND {
        problems.push(format!(
            "card: sits {off_background:.1} from the background — it would not be seen"
        ));
    }

    let well_off_background = separation(well, background);
    if well_off_background < PLANE_FROM_PLANE {
        problems.push(format!(
            "well: sits {well_off_background:.1} from the background — it would not be seen"
        ));
    }

    let between = separation(card, well);
    if between < PLANE_FROM_PLANE {
        problems.push(format!(
            "planes: card and well sit {between:.1} apart — they read as one"
        ));
    }

    if crossed(background, card, well) {
        problems.push(format!(
            "depth: the card ({off_background:.1} off the background) is further out than the well ({well_off_background:.1}) — the planes are inverted"
        ));
    }

    problems
}

/// Both planes on the same side of the background is fine on a light theme, where nothing fits
/// above it; the card being the further of the two is not.
fn crossed(background: Rgb, card: Rgb, well: Rgb) -> bool {
    let ground = tone_of(background);
    let same_side = (tone_of(card) - ground).signum() == (tone_of(well) - ground).signum();

    same_side && separation(card, background) > separation(well, background)
}

fn collisions(roles: &Roles) -> Vec<String> {
    [
        (
            Role::Focus,
            Role::Primary,
            "focus reads as a primary element",
        ),
        (
            Role::Disabled,
            Role::Muted,
            "a disabled element reads as a muted one",
        ),
    ]
    .into_iter()
    .filter_map(|(one, other, consequence)| {
        let gap = separation(roles.color(one), roles.color(other));
        (gap < ROLE_FROM_ROLE)
            .then(|| format!("roles: {one:?} sits {gap:.1} from {other:?} — {consequence}",))
    })
    .collect()
}

fn content(roles: &Roles, theme: &Theme) -> Vec<String> {
    let mut problems = Vec::new();

    for role in Role::ALL {
        let ground = roles.color(role);
        let main_content = roles.on(role);
        let gap = |level| separation(roles.content(role, level), ground);
        let (main, secondary, disabled) = (
            gap(Level::Primary),
            gap(Level::Secondary),
            gap(Level::Disabled),
        );

        if main < CONTENT_FROM_GROUND {
            problems.push(format!(
                "text on {role:?}: main text sits {main:.1} from its ground — it would not read"
            ));
        }
        if !(main > secondary && secondary > disabled) {
            problems.push(format!(
                "text on {role:?}: the three weights do not step back ({main:.1}, {secondary:.1}, {disabled:.1})"
            ));
        }
        if disabled < DISABLED_FROM_GROUND {
            problems.push(format!(
                "text on {role:?}: disabled text sits {disabled:.1} from its ground — it disappears"
            ));
        }

        // A theme's own foreground can carry any hue it wants; only content the library had to
        // invent is asked to stay in its ground's family.
        if chroma_of(ground) >= CHROMATIC_GROUND && !is_declared(main_content, theme) {
            let drift = hue_gap(main_content, ground);
            if drift > MAX_HUE_DRIFT {
                problems.push(format!(
                    "text on {role:?}: invented main text drifts {drift:.0} in hue from its ground — it reads as an unrelated color"
                ));
            }
        }
    }

    problems
}

fn is_declared(content: Rgb, theme: &Theme) -> bool {
    let colors = &theme.colors;
    [
        colors.foreground,
        colors.bright_foreground,
        colors.light_foreground,
        colors.dark_foreground,
    ]
    .contains(&content)
}

fn theme_name(path: &Path, fallback: &str) -> String {
    path.parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| fallback.to_owned())
}
