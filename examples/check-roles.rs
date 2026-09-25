//! Checks every installed theme resolves to readable roles:
//! `cargo run --example check-roles -- <colors.toml>...`

use omarchy_lib_theme::{Role, Roles, read_theme};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut worst = f64::MAX;
    let mut failures = 0;

    for arg in std::env::args().skip(1) {
        let path = PathBuf::from(&arg);
        let name = path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| arg.clone());

        let theme = match read_theme(&path) {
            Ok(theme) => theme,
            Err(error) => {
                println!("FAIL {name:<18} {error}");
                failures += 1;
                continue;
            }
        };

        let roles = Roles::resolve(&theme);
        let mut lowest = f64::MAX;
        let mut lowest_role = Role::Background;
        for role in Role::ALL {
            let gap = omarchy_lib_theme::separation(roles.on(role), roles.color(role));
            if gap < lowest {
                lowest = gap;
                lowest_role = role;
            }
        }

        let mode = if theme.mode.is_dark() {
            "dark"
        } else {
            "light"
        };
        let verdict = if lowest >= 40.0 { "ok  " } else { "FAIL" };
        if lowest < 40.0 {
            failures += 1;
        }
        worst = worst.min(lowest);
        println!("{verdict} {name:<18} {mode:<5} worst {lowest:>5.1}  ({lowest_role:?})");
    }

    println!("-----");
    println!("worst separation across all themes: {worst:.1}   failures: {failures}");
    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
