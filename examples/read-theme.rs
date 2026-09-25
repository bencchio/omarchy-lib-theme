//! Shows the theme the library reads: `cargo run --example read-theme [path to colors.toml]`.

use std::path::PathBuf;
use std::process::ExitCode;

use omarchy_lib_theme::{
    Level, Ramp, Rgb, Role, Roles, Theme, read_active_theme, read_theme, separation,
};

fn main() -> ExitCode {
    let source = std::env::args().nth(1);
    let theme = match &source {
        Some(path) => read_theme(&PathBuf::from(path)),
        None => read_active_theme(),
    };

    match theme {
        Ok(theme) => {
            show(&theme, source.as_deref().unwrap_or("active theme"));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not read the theme: {error}");
            ExitCode::FAILURE
        }
    }
}

fn show(theme: &Theme, source: &str) {
    let colors = &theme.colors;
    println!("{source}  ({:?})\n", theme.mode);

    let groups: [(&str, &[(&str, Rgb)]); 4] = [
        (
            "base",
            &[
                ("accent", colors.accent),
                ("selection", colors.selection),
                ("muted", colors.muted),
            ],
        ),
        (
            "backgrounds",
            &[
                ("darker_background", colors.darker_background),
                ("dark_background", colors.dark_background),
                ("background", colors.background),
                ("lighter_background", colors.lighter_background),
            ],
        ),
        (
            "foregrounds",
            &[
                ("dark_foreground", colors.dark_foreground),
                ("light_foreground", colors.light_foreground),
                ("foreground", colors.foreground),
                ("bright_foreground", colors.bright_foreground),
            ],
        ),
        (
            "palette",
            &[
                ("red", colors.red),
                ("yellow", colors.yellow),
                ("green", colors.green),
                ("cyan", colors.cyan),
                ("blue", colors.blue),
                ("magenta", colors.magenta),
            ],
        ),
    ];

    for (title, entries) in groups {
        println!("{title}");
        for (name, color) in entries {
            println!("{}", color_row(name, *color));
        }
        println!();
    }

    show_roles(theme);
}

fn show_roles(theme: &Theme) {
    let roles = Roles::resolve(theme);

    println!("\nroles");
    println!(
        "  {:<18} {:<9} {:<5}  {:<18}  tonal ramp",
        "", "color", "gap", "main  second  off"
    );
    for role in Role::ALL {
        let color = roles.color(role);
        let samples: String = Level::ALL
            .iter()
            .map(|level| sample(color, roles.content(role, *level)))
            .collect();

        println!(
            "  {:<18} {:<9} {:>5.1}  {samples}  {}",
            format!("{role:?}"),
            color.to_string(),
            separation(roles.on(role), color),
            ramp(color),
        );
    }
}

fn color_row(name: &str, color: Rgb) -> String {
    format!(
        "  {} {name:<18} {:<9} {}",
        swatch(color),
        color.to_string(),
        ramp(color)
    )
}

fn ramp(color: Rgb) -> String {
    let ramp = Ramp::of(color);
    (0..=100)
        .step_by(10)
        .map(|step| step_swatch(ramp.tone(step)))
        .collect()
}

fn sample(ground: Rgb, content: Rgb) -> String {
    format!(
        "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m  Aa  \x1b[0m",
        ground.red, ground.green, ground.blue, content.red, content.green, content.blue
    )
}

fn step_swatch(color: Rgb) -> String {
    format!(
        "\x1b[48;2;{};{};{}m  \x1b[0m",
        color.red, color.green, color.blue
    )
}

fn swatch(color: Rgb) -> String {
    format!(
        "\x1b[48;2;{};{};{}m    \x1b[0m",
        color.red, color.green, color.blue
    )
}
