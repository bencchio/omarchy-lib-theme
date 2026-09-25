//! The library must stay free of explicit theme references outside its tests: a theme is picked by
//! the active Omarchy theme, never by name. Test apps may name themes; the library contract may not.

use std::fs;
use std::path::{Path, PathBuf};

/// Explicit Omarchy theme slugs. A match outside a `#[cfg(test)]` module fails the guard.
const THEME_SLUGS: &[&str] = &[
    "everforest",
    "flexoki",
    "catppuccin",
    "gruvbox",
    "kanagawa",
    "nord",
    "tokyo-night",
    "rose-pine",
    "miasma",
    "osaka-jade",
    "matte-black",
    "vantablack",
    "ethereal",
    "hackerman",
    "lumon",
    "lupine",
    "last-horizon",
    "retro-82",
    "ristretto",
    "solitude",
];

fn src_roots() -> Vec<PathBuf> {
    let manifest = env!("CARGO_MANIFEST_DIR");
    vec![Path::new(manifest).join("src")]
}

fn rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).expect("the source root can be read");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// True when the line is inside a `#[cfg(test)] mod tests { ... }` block, tracked by brace depth.
fn inside_test_module(lines: &[&str], index: usize) -> bool {
    let mut depth: i64 = 0;
    let mut in_tests = false;
    let mut awaiting_cfg = false;

    for line in lines.iter().take(index) {
        let line = line.trim_start();
        if line.starts_with("#[cfg(test)]") {
            awaiting_cfg = true;
            continue;
        }
        if awaiting_cfg && line.starts_with("mod ") {
            in_tests = true;
            awaiting_cfg = false;
        } else {
            awaiting_cfg = false;
        }
        depth += line.bytes().filter(|b| *b == b'{').count() as i64;
        depth -= line.bytes().filter(|b| *b == b'}').count() as i64;
        if in_tests && depth == 0 {
            in_tests = false;
        }
    }
    in_tests
}

#[test]
fn the_library_carries_no_explicit_theme_reference() {
    let mut files = Vec::new();
    for root in src_roots() {
        rs_files(&root, &mut files);
    }

    let mut violations = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path).expect("a source file can be read");
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if inside_test_module(&lines, i) {
                continue;
            }
            if let Some(slug) = THEME_SLUGS.iter().find(|slug| line.contains(**slug)) {
                violations.push(format!("{}:{} mentions {slug}", path.display(), i + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "the library references explicit themes outside its tests:\n{}",
        violations.join("\n")
    );
}
