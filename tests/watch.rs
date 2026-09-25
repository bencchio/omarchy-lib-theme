//! The watcher against the ways a theme file actually changes on disk.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use omarchy_lib_theme::{Mode, Theme, ThemeWatcher};

/// Long enough for inotify to deliver and the reader to settle, short enough to stay a test.
const PATIENCE: Duration = Duration::from_secs(5);

/// Long enough for a change to have been noticed, if one were coming at all.
const QUIET: Duration = Duration::from_millis(400);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is set before 1970")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("omarchy-lib-theme-{label}-{stamp}"));
        fs::create_dir_all(&path).expect("the temporary directory can be created");
        Self { path }
    }

    fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // A leftover directory under /tmp is not worth failing a test that already passed.
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(path).expect("the fixture is readable")
}

fn wait_for_change(watcher: &ThemeWatcher) -> Option<Theme> {
    let deadline = Instant::now() + PATIENCE;
    while Instant::now() < deadline {
        if let Some(theme) = watcher.changed() {
            return Some(theme);
        }
        thread::sleep(Duration::from_millis(10));
    }
    None
}

fn start(label: &str) -> (TempDir, PathBuf, ThemeWatcher) {
    let directory = TempDir::new(label);
    let theme = directory.file("colors.toml");
    fs::write(&theme, fixture("everforest.toml")).expect("the theme can be written");
    let watcher = ThemeWatcher::watch(&theme).expect("the watcher starts on a valid theme");
    (directory, theme, watcher)
}

#[test]
fn starts_on_the_theme_that_is_there() {
    let (_directory, _theme, watcher) = start("start");

    assert_eq!(watcher.current().mode, Mode::Dark);
    assert!(watcher.changed().is_none());
}

#[test]
fn refuses_to_start_without_a_theme() {
    let directory = TempDir::new("absent");

    let error = ThemeWatcher::watch(&directory.file("colors.toml"));

    assert!(error.is_err());
}

#[test]
fn notices_the_file_rewritten_in_place() {
    let (_directory, theme, watcher) = start("rewrite");

    fs::write(&theme, fixture("flexoki-light.toml")).expect("the theme can be rewritten");

    let arrived = wait_for_change(&watcher).expect("the rewrite is noticed");
    assert_eq!(arrived.mode, Mode::Light);
    assert_eq!(watcher.current().mode, Mode::Light);
}

#[test]
fn notices_the_file_replaced_by_a_rename() {
    let (directory, theme, watcher) = start("rename");

    let incoming = directory.file("colors.toml.incoming");
    fs::write(&incoming, fixture("flexoki-light.toml")).expect("the new theme can be staged");
    fs::rename(&incoming, &theme).expect("the new theme can be renamed over the old one");

    let arrived = wait_for_change(&watcher).expect("the rename is noticed");
    assert_eq!(arrived.mode, Mode::Light);
}

#[test]
fn keeps_the_last_valid_theme_when_a_reload_fails() {
    let (_directory, theme, watcher) = start("invalid");

    fs::write(&theme, "mode = \"dark\"\naccent = \n[[[").expect("the theme can be corrupted");
    thread::sleep(QUIET);

    assert!(watcher.changed().is_none());
    assert_eq!(watcher.current().mode, Mode::Dark);
    assert_eq!(
        watcher.current().colors.accent,
        omarchy_lib_theme::Rgb::parse("accent", "#7fbbb3").unwrap()
    );
}

#[test]
fn recovers_once_the_theme_is_whole_again() {
    let (_directory, theme, watcher) = start("recover");

    fs::write(&theme, "not a theme at all").expect("the theme can be corrupted");
    thread::sleep(QUIET);
    fs::write(&theme, fixture("flexoki-light.toml")).expect("the theme can be restored");

    let arrived = wait_for_change(&watcher).expect("the restored theme is noticed");
    assert_eq!(arrived.mode, Mode::Light);
}

#[test]
fn hands_over_the_newest_of_several_changes() {
    let (_directory, theme, watcher) = start("newest");

    fs::write(&theme, fixture("flexoki-light.toml")).expect("the theme can be rewritten");
    thread::sleep(Duration::from_millis(150));
    fs::write(&theme, fixture("everforest.toml")).expect("the theme can be rewritten");

    let deadline = Instant::now() + PATIENCE;
    let mut latest = None;
    while Instant::now() < deadline {
        if let Some(theme) = watcher.changed() {
            latest = Some(theme);
        }
        if latest
            .as_ref()
            .is_some_and(|theme: &Theme| theme.mode == Mode::Dark)
        {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    let latest = latest.expect("at least one change is noticed");
    assert_eq!(latest, watcher.current());
    assert_eq!(latest.mode, Mode::Dark, "the newest theme won");
}

/// How Omarchy actually switches theme: the whole directory holding `colors.toml` is removed and
/// another one renamed into its place, so the directory the watcher started on stops existing.
fn swap_theme_directory(root: &Path, contents: &str) {
    let staging = root.join("next-theme");
    let current = root.join("theme");

    fs::create_dir_all(&staging).expect("the staging directory can be created");
    fs::write(staging.join("colors.toml"), contents).expect("the next theme can be staged");
    fs::remove_dir_all(&current).expect("the current theme directory can be removed");
    fs::rename(&staging, &current).expect("the next theme directory can be moved into place");
}

#[test]
fn keeps_noticing_after_the_theme_directory_is_replaced() {
    let root = TempDir::new("directory-swap");
    let theme = root.path.join("theme").join("colors.toml");
    fs::create_dir_all(theme.parent().expect("the theme file sits in a directory"))
        .expect("the theme directory can be created");
    fs::write(&theme, fixture("everforest.toml")).expect("the theme can be written");

    let watcher = ThemeWatcher::watch(&theme).expect("the watcher starts on a valid theme");

    swap_theme_directory(&root.path, &fixture("flexoki-light.toml"));
    let first = wait_for_change(&watcher).expect("the first theme switch is noticed");
    assert_eq!(first.mode, Mode::Light);

    swap_theme_directory(&root.path, &fixture("everforest.toml"));
    let second = wait_for_change(&watcher).expect("the second theme switch is noticed");
    assert_eq!(second.mode, Mode::Dark);
}

#[test]
fn reports_nothing_when_the_theme_is_rewritten_unchanged() {
    let (_directory, theme, watcher) = start("identical");

    fs::write(&theme, fixture("everforest.toml")).expect("the theme can be rewritten");
    thread::sleep(QUIET);

    assert!(watcher.changed().is_none());
}
