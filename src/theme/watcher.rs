//! Watching the theme on disk and handing the application each new one it can use.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::Theme;
use super::error::ThemeError;
use super::reader::{default_theme_path, read_theme};

/// Switching theme rewrites the file, which can be read back while it is still half-written, so a
/// failed read is retried for a moment before the change is given up on.
const READ_ATTEMPTS: usize = 5;
const RETRY_DELAY: Duration = Duration::from_millis(20);

/// Follows the theme file and keeps the last one that could be read.
///
/// The application decides when to collect: nothing is pushed onto its threads, and neither
/// [`current`](Self::current) nor [`changed`](Self::changed) blocks.
pub struct ThemeWatcher {
    state: Arc<Mutex<State>>,
    /// Dropping the watcher stops the notifications, so it is held for as long as this is alive.
    _watcher: RecommendedWatcher,
}

struct State {
    current: Theme,
    changed: Option<Theme>,
}

impl ThemeWatcher {
    pub fn active() -> Result<Self, ThemeError> {
        Self::watch(&default_theme_path())
    }

    /// Fails when the theme cannot be read right now: starting without colors is an error the
    /// caller has to see. Once running, a failed reload never surfaces — the last valid theme
    /// stays in force instead.
    pub fn watch(path: &Path) -> Result<Self, ThemeError> {
        Self::watch_with(path, |_| {})
    }

    /// Same as [`watch`](Self::watch), plus `on_change`, called from notify's own thread right
    /// after a valid change lands — the hook `api::ThemeWatch` uses to signal a change through a
    /// file descriptor without polling this watcher itself.
    pub(crate) fn watch_with(
        path: &Path,
        mut on_change: impl FnMut(&Theme) + Send + 'static,
    ) -> Result<Self, ThemeError> {
        let path = path.to_owned();
        let directory = directory_of(&path).to_owned();
        let state = Arc::new(Mutex::new(State {
            current: read_theme(&path)?,
            changed: None,
        }));

        let updates = Arc::clone(&state);
        let watched = path.clone();
        let watched_directory = directory.clone();
        let mut watcher = notify::recommended_watcher(move |event| {
            if let Ok(event) = event
                && touches(&event, &watched, &watched_directory)
                && let Ok(theme) = read_settled(&watched)
            {
                let mut state = lock(&updates);
                if theme != state.current {
                    state.current = theme.clone();
                    state.changed = Some(theme.clone());
                    drop(state);
                    on_change(&theme);
                }
            }
        })
        .map_err(ThemeError::Watch)?;

        watcher
            .watch(&directory, RecursiveMode::NonRecursive)
            .map_err(ThemeError::Watch)?;

        if let Some(above) = above(&directory) {
            watcher
                .watch(above, RecursiveMode::NonRecursive)
                .map_err(ThemeError::Watch)?;
        }

        Ok(Self {
            state,
            _watcher: watcher,
        })
    }

    /// The theme in force.
    pub fn current(&self) -> Theme {
        lock(&self.state).current.clone()
    }

    /// The new theme if one arrived since the last call.
    ///
    /// Only the newest survives: when several changes land between two calls the ones in between
    /// are dropped, since a theme nobody drew yet has nothing to offer.
    pub fn changed(&self) -> Option<Theme> {
        lock(&self.state).changed.take()
    }
}

/// The theme directory is watched rather than the file itself: switching theme can replace
/// `colors.toml` by renaming another file over it, and a watch on the old file would not see that.
fn directory_of(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

/// The directory above the theme directory, which is also watched. Omarchy switches theme by
/// removing the whole theme directory and renaming another one into its place, and a watch follows
/// the inode rather than the path: the watch on the theme directory dies with it, and only the one
/// above still reports the swap. `None` where there is nothing above to watch.
fn above(directory: &Path) -> Option<&Path> {
    directory
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

/// Whether an event concerns the theme: the file changing inside the theme directory, or the theme
/// directory itself being replaced within the directory above it.
///
/// Being opened or read is not a change, and reading the theme back in answer to it would produce
/// the very event that triggered the read — a loop that never settles.
fn touches(event: &notify::Event, file: &Path, directory: &Path) -> bool {
    if matches!(event.kind, EventKind::Access(_)) {
        return false;
    }

    event.paths.iter().any(|path: &PathBuf| {
        let name = path.file_name();
        name.is_some() && (name == file.file_name() || name == directory.file_name())
    })
}

fn read_settled(path: &Path) -> Result<Theme, ThemeError> {
    let mut result = read_theme(path);
    for _ in 1..READ_ATTEMPTS {
        if result.is_ok() {
            break;
        }
        thread::sleep(RETRY_DELAY);
        result = read_theme(path);
    }
    result
}

/// A panic while holding the lock would poison it, but the guarded state is a theme and a slot,
/// both always whole, so recovering keeps the watcher alive instead of spreading the panic.
fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
