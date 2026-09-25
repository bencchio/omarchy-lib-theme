//! Walks the installed themes with the keyboard:
//! `cargo run --example browse-themes -- /usr/share/omarchy/themes/*/colors.toml`

use std::io::{Read, Write, stdin, stdout};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use omarchy_lib_theme::{Level, Ramp, Rgb, Role, Roles, Theme, read_theme, separation};

fn main() -> ExitCode {
    let paths: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    if paths.is_empty() {
        eprintln!("usage: browse-themes <path to colors.toml>...");
        return ExitCode::FAILURE;
    }

    let themes: Vec<(String, Theme)> = paths
        .iter()
        .filter_map(|path| match read_theme(path) {
            Ok(theme) => Some((name_of(path), theme)),
            Err(error) => {
                eprintln!("skipping {}: {error}", path.display());
                None
            }
        })
        .collect();

    if themes.is_empty() {
        eprintln!("none of the given paths holds a readable theme");
        return ExitCode::FAILURE;
    }

    let Some(terminal) = RawTerminal::enter() else {
        eprintln!("this terminal cannot be put in raw mode");
        return ExitCode::FAILURE;
    };

    browse(&themes);
    drop(terminal);
    ExitCode::SUCCESS
}

fn browse(themes: &[(String, Theme)]) {
    let mut current = 0;
    let mut keys = stdin().lock().bytes();

    loop {
        let (name, theme) = &themes[current];
        draw(name, theme, current + 1, themes.len());

        match keys.next() {
            Some(Ok(key)) => match Key::of(key, &mut keys) {
                Key::Previous => current = (current + themes.len() - 1) % themes.len(),
                Key::Next => current = (current + 1) % themes.len(),
                Key::Quit => break,
                Key::Ignored => {}
            },
            _ => break,
        }
    }

    print!("\x1b[2J\x1b[H");
    let _ = stdout().flush();
}

enum Key {
    Previous,
    Next,
    Quit,
    Ignored,
}

impl Key {
    /// Arrows arrive as three bytes, so the two that follow the escape are taken from the same
    /// stream before deciding what was pressed.
    fn of(first: u8, rest: &mut impl Iterator<Item = std::io::Result<u8>>) -> Self {
        match first {
            b'q' | 0x03 => Self::Quit,
            b'k' => Self::Previous,
            b'j' => Self::Next,
            0x1b => match (rest.next(), rest.next()) {
                (Some(Ok(b'[')), Some(Ok(b'A' | b'D'))) => Self::Previous,
                (Some(Ok(b'[')), Some(Ok(b'B' | b'C'))) => Self::Next,
                _ => Self::Ignored,
            },
            _ => Self::Ignored,
        }
    }
}

fn draw(name: &str, theme: &Theme, position: usize, total: usize) {
    let roles = Roles::resolve(theme);
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };

    let mut screen = String::from("\x1b[2J\x1b[H");
    screen.push_str(&format!("{name}  ({mode})   {position}/{total}\r\n\r\n"));
    screen.push_str(&format!(
        "  {:<18} {:<9} {:<5}  {:<18}  tonal ramp\r\n",
        "", "color", "gap", "main  second  off"
    ));

    for role in Role::ALL {
        let color = roles.color(role);
        let samples: String = Level::ALL
            .iter()
            .map(|level| sample(color, roles.content(role, *level)))
            .collect();

        screen.push_str(&format!(
            "  {:<18} {:<9} {:>5.1}  {samples}  {}\r\n",
            format!("{role:?}"),
            color.to_string(),
            separation(roles.on(role), color),
            ramp(color),
        ));
    }

    screen.push_str("\r\n  ←/→ or j/k to move, q to quit\r\n");

    print!("{screen}");
    let _ = stdout().flush();
}

fn ramp(color: Rgb) -> String {
    let ramp = Ramp::of(color);
    (0..=100)
        .step_by(10)
        .map(|step| {
            let tone = ramp.tone(step);
            format!(
                "\x1b[48;2;{};{};{}m  \x1b[0m",
                tone.red, tone.green, tone.blue
            )
        })
        .collect()
}

fn sample(ground: Rgb, content: Rgb) -> String {
    format!(
        "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m  Aa  \x1b[0m",
        ground.red, ground.green, ground.blue, content.red, content.green, content.blue
    )
}

fn name_of(path: &Path) -> String {
    path.parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// The terminal keeps whatever mode it was left in, so restoring it belongs to a value that is
/// dropped on every way out, an unwinding panic included.
struct RawTerminal;

impl RawTerminal {
    fn enter() -> Option<Self> {
        stty(&["raw", "-echo"]).then_some(Self)
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        stty(&["sane"]);
    }
}

fn stty(arguments: &[&str]) -> bool {
    Command::new("stty")
        .args(arguments)
        .stdin(Stdio::inherit())
        .status()
        .is_ok_and(|status| status.success())
}
