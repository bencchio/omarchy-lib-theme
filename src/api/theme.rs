//! Watching the active theme and keeping its resolved palette ready for on-demand pickup.
//!
//! Accessible like [`ThemeWatcher`] and [`Treatment`](crate::modes::Treatment), but outside the
//! promoted v1 contract (`PaletteSeed`/`Palette` — see `api/palette.rs`).

use std::io::Write;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::theme::{Theme, ThemeError, ThemeWatcher, default_theme_path, read_theme};

use super::palette::{Palette, PaletteSeed};

/// Follows the active theme and keeps the palette it resolves to, ready for on-demand pickup.
///
/// Change notification runs through a file descriptor rather than polling: [`signal_fd`](Self::signal_fd)
/// gives a fd the caller integrates into its own event loop, readable whenever a valid change
/// landed. [`changed`](Self::changed) then hands over the new [`Palette`], already resolved.
pub struct ThemeWatch {
    watcher: ThemeWatcher,
    signal: UnixStream,
    state: Arc<Mutex<State>>,
}

struct State {
    current: Palette,
    changed: Option<Palette>,
}

impl ThemeWatch {
    /// Watches the system's active Omarchy theme.
    pub fn active() -> Result<Self, ThemeError> {
        Self::watch(&default_theme_path())
    }

    /// Watches the Omarchy-style theme at `path`.
    pub fn watch(path: &Path) -> Result<Self, ThemeError> {
        let (signal, mut notifier) = UnixStream::pair().map_err(ThemeError::Signal)?;
        signal.set_nonblocking(true).map_err(ThemeError::Signal)?;
        notifier.set_nonblocking(true).map_err(ThemeError::Signal)?;

        let initial = read_theme(path)?;
        let state = Arc::new(Mutex::new(State {
            current: PaletteSeed::from_theme(&initial).extend(),
            changed: None,
        }));

        let hook_state = Arc::clone(&state);
        let watcher = ThemeWatcher::watch_with(path, move |theme: &Theme| {
            let palette = PaletteSeed::from_theme(theme).extend();
            let mut state = lock(&hook_state);
            state.current = palette.clone();
            state.changed = Some(palette);
            let _ = notifier.write(&[1]);
        })?;

        Ok(Self {
            watcher,
            signal,
            state,
        })
    }

    /// The underlying watcher, for consumers that need the raw [`Theme`].
    ///
    /// No longer needed inside this crate: `current`/`changed` already hand out a [`Palette`],
    /// and [`Palette::represented`](crate::api::Palette) reaches every treatment from there.
    /// Kept for now — removing it closes the same parallel resolution path
    /// `Plegar MODES en el pipeline` holds until the real-app validation.
    #[deprecated(note = "read the Theme through current()/changed()'s Palette instead")]
    pub fn watcher(&self) -> &ThemeWatcher {
        &self.watcher
    }

    /// The palette resolved from the theme in force.
    pub fn current(&self) -> Palette {
        lock(&self.state).current.clone()
    }

    /// The newly resolved palette if a change arrived since the last call. Only the newest
    /// survives, same as [`ThemeWatcher::changed`].
    pub fn changed(&self) -> Option<Palette> {
        lock(&self.state).changed.take()
    }

    /// A file descriptor the caller integrates into its own event loop: readable whenever a valid
    /// change landed. Reading from it never blocks; draining it fully before the next wait is the
    /// caller's job.
    pub fn signal_fd(&self) -> RawFd {
        self.signal.as_raw_fd()
    }
}

/// A panic while holding the lock would poison it, but the guarded state is a palette and a slot,
/// both always whole, so recovering keeps the watch alive instead of spreading the panic.
fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::{Duration, Instant};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn resolves_the_initial_palette() {
        let watch = ThemeWatch::watch(&fixture("everforest.toml")).unwrap();
        assert!(watch.current().dark());
    }

    #[test]
    fn reports_a_missing_theme() {
        match ThemeWatch::watch(&fixture("does-not-exist.toml")) {
            Err(ThemeError::NotFound(_)) => {}
            _ => panic!("expected ThemeError::NotFound"),
        }
    }

    #[test]
    fn signals_and_resolves_a_change() {
        let source = fixture("everforest.toml");
        let path = std::env::temp_dir().join(format!(
            "omarchy-lib-theme-watch-{}-{}.toml",
            std::process::id(),
            line!()
        ));
        std::fs::copy(&source, &path).unwrap();

        let watch = ThemeWatch::watch(&path).unwrap();
        assert!(watch.changed().is_none());

        std::fs::copy(fixture("flexoki-light.toml"), &path).unwrap();

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut byte = [0u8; 1];
        let mut signalled = false;
        let mut signal = &watch.signal;
        while Instant::now() < deadline {
            if signal.read(&mut byte).is_ok() {
                signalled = true;
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        assert!(signalled, "no change signalled on the fd in time");

        let changed = watch
            .changed()
            .expect("a resolved palette after the signal");
        assert!(!changed.dark());
        assert!(watch.changed().is_none());

        std::fs::remove_file(&path).ok();
    }
}
