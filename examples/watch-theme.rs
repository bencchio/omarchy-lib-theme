//! Follows the active theme and reports every change:
//! `cargo run --example watch-theme` — then switch theme in Omarchy.

use std::thread;
use std::time::Duration;

use omarchy_lib_theme::{Role, Roles, Theme, ThemeWatcher};

fn main() {
    let watcher = match ThemeWatcher::active() {
        Ok(watcher) => watcher,
        Err(error) => {
            eprintln!("could not watch the theme: {error}");
            return;
        }
    };

    println!("watching the active theme — switch theme in Omarchy, ctrl-c to stop\n");
    report("current", &watcher.current());

    loop {
        if let Some(theme) = watcher.changed() {
            report("changed", &theme);
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn report(label: &str, theme: &Theme) {
    let roles = Roles::resolve(theme);
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };

    print!("{label:<8} {mode:<6}");
    for role in Role::ALL {
        let color = roles.color(role);
        print!(
            " \x1b[48;2;{};{};{}m  \x1b[0m",
            color.red, color.green, color.blue
        );
    }
    println!("  {}", roles.color(Role::Background));
}
