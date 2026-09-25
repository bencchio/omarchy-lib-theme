//! The C ABI: a separate contract from the Rust API's v1 (`docs/API.md`), not an extension of it —
//! see `BRIDGE.md` for the usage flow, the threading contract and what each treatment does.
//!
//! `include/omarchy_lib_theme.h` is a hand-written mirror of this file and has to be kept in sync by
//! hand, since no generator produces it.

use std::ffi::{CStr, c_char, c_int};
use std::path::PathBuf;
use std::ptr;

use super::palette::Palette;
use super::theme::ThemeWatch;
use crate::color::{Deposit, Level, Role, State};
use crate::modes::Treatment;
use crate::theme::{Family, default_theme_path};

/// An RGB color, one byte per channel — the same precision `colors.toml` declares.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmarchyLibThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// One role: its own color, and the three weights of content that read on it.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmarchyLibThemeRole {
    pub color: OmarchyLibThemeColor,
    pub content_main: OmarchyLibThemeColor,
    pub content_secondary: OmarchyLibThemeColor,
    pub content_disabled: OmarchyLibThemeColor,
}

/// How many roles a snapshot carries — `Role::ALL.len()`, restated here so the header does not
/// need to reach back into Rust to size its array.
pub const OMARCHY_LIB_THEME_ROLE_COUNT: usize = 10;

/// One shade of a color: the shade itself and the content that reads on it.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmarchyLibThemeShade {
    pub color: OmarchyLibThemeColor,
    pub content: OmarchyLibThemeColor,
}

/// One color of the theme, as the family of three it reads as: the color itself, its dark shade and
/// its bright one, in that order.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmarchyLibThemeColorFamily {
    pub original: OmarchyLibThemeShade,
    pub dark: OmarchyLibThemeShade,
    pub bright: OmarchyLibThemeShade,
    pub declared: bool,
}

/// How many colors a snapshot carries, each as a family of three shades.
pub const OMARCHY_LIB_THEME_COLOR_COUNT: usize = 6;

/// Every role resolved against the theme in force, in the fixed order [`omarchy_lib_theme_role_name`]
/// names, and every color the theme declares, in the order [`omarchy_lib_theme_color_name`] names.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmarchyLibThemeSnapshot {
    pub roles: [OmarchyLibThemeRole; OMARCHY_LIB_THEME_ROLE_COUNT],
    pub colors: [OmarchyLibThemeColorFamily; OMARCHY_LIB_THEME_COLOR_COUNT],
    pub dark: bool,
}

/// An opaque handle to a running [`ThemeWatch`]. Only ever seen through a pointer on the C side.
pub struct OmarchyLibThemeWatcher(ThemeWatch);

fn to_color(rgb: crate::theme::Rgb) -> OmarchyLibThemeColor {
    OmarchyLibThemeColor {
        r: rgb.red,
        g: rgb.green,
        b: rgb.blue,
    }
}

/// How many treatments [`omarchy_lib_theme_treatment_name`] can name — `Treatment::ALL.len()`.
/// Only the C side loops over this to build a mode switcher; Rust itself iterates
/// `Treatment::ALL` directly, so this constant otherwise looks unused outside tests.
#[allow(dead_code)]
pub const OMARCHY_LIB_THEME_TREATMENT_COUNT: usize = 6;

/// The treatment at `index`, in the same order [`omarchy_lib_theme_treatment_name`] names. An
/// out-of-range index is not a caller error worth failing over — it falls back to `Original`,
/// documented here rather than left to panic across the FFI boundary.
fn treatment_at(index: usize) -> Treatment {
    Treatment::ALL
        .get(index)
        .copied()
        .unwrap_or(Treatment::Original)
}

/// How many deposits [`omarchy_lib_theme_deposit_name`] can name — `Deposit::ALL.len()`. Like
/// [`OMARCHY_LIB_THEME_TREATMENT_COUNT`], only the C side loops over it.
#[allow(dead_code)]
pub const OMARCHY_LIB_THEME_DEPOSIT_COUNT: usize = 5;

/// How many states [`omarchy_lib_theme_state_name`] can name — `State::ALL.len()`.
#[allow(dead_code)]
pub const OMARCHY_LIB_THEME_STATE_COUNT: usize = 5;

/// The deposit at `index`, in the same order [`omarchy_lib_theme_deposit_name`] names. An
/// out-of-range index is not a caller error worth failing over — it falls back to `Fill`,
/// documented here rather than left to panic across the FFI boundary.
fn deposit_at(index: usize) -> Deposit {
    Deposit::ALL.get(index).copied().unwrap_or(Deposit::Fill)
}

/// The state at `index`, in the same order [`omarchy_lib_theme_state_name`] names. An out-of-range
/// index falls back to `Rest`, the neutral situation.
fn state_at(index: usize) -> State {
    State::ALL.get(index).copied().unwrap_or(State::Rest)
}

/// The role at `index`, read against [`SNAPSHOT_ROLES`] — the same fixed order the snapshot and
/// [`omarchy_lib_theme_role_name`] use, so a Rust-only role added later cannot shift what an
/// index means. An out-of-range index falls back to `Background`.
fn role_at(index: usize) -> Role {
    SNAPSHOT_ROLES
        .get(index)
        .copied()
        .unwrap_or(Role::Background)
}

/// The roles a snapshot carries, in the fixed order [`OmarchyLibThemeSnapshot::roles`] uses. Kept
/// apart from [`Role::ALL`] so that a role added to Rust later cannot silently change the C
/// layout — the snapshot only grows when this array does, on purpose.
const SNAPSHOT_ROLES: [Role; OMARCHY_LIB_THEME_ROLE_COUNT] = [
    Role::Background,
    Role::Elevated,
    Role::Recessed,
    Role::Border,
    Role::Primary,
    Role::Selection,
    Role::Muted,
    Role::Error,
    Role::Focus,
    Role::Disabled,
];

/// Builds the snapshot `palette` resolves to. Kept apart from the `unsafe` boundary functions so
/// it can be tested directly, with an ordinary [`Palette`] and no raw pointers involved. Pure
/// marshalling: every color and every guarantee behind it already came from `palette`.
fn snapshot(palette: &Palette) -> OmarchyLibThemeSnapshot {
    let roles = SNAPSHOT_ROLES.map(|role| OmarchyLibThemeRole {
        color: to_color(palette.color(role)),
        content_main: to_color(palette.content(role, Level::Primary)),
        content_secondary: to_color(palette.content(role, Level::Secondary)),
        content_disabled: to_color(palette.content(role, Level::Disabled)),
    });

    let shade = |color: crate::theme::Rgb| OmarchyLibThemeShade {
        color: to_color(color),
        content: to_color(palette.on_color(color)),
    };
    let colors = Family::ALL.map(|family| {
        let shades = palette.family(family);
        OmarchyLibThemeColorFamily {
            original: shade(shades.color),
            dark: shade(shades.dark),
            bright: shade(shades.bright),
            declared: true,
        }
    });

    OmarchyLibThemeSnapshot {
        roles,
        colors,
        dark: palette.dark(),
    }
}

/// The display name of the color at `index`, in the same order [`OmarchyLibThemeSnapshot::colors`] uses.
///
/// A static string, owned by the library — never freed by the caller. Returns null for an index at
/// or past [`OMARCHY_LIB_THEME_COLOR_COUNT`].
#[unsafe(no_mangle)]
pub extern "C" fn omarchy_lib_theme_color_name(index: usize) -> *const c_char {
    const NAMES: [&CStr; OMARCHY_LIB_THEME_COLOR_COUNT] =
        [c"Red", c"Green", c"Yellow", c"Blue", c"Cyan", c"Magenta"];

    NAMES.get(index).map_or(ptr::null(), |name| name.as_ptr())
}

/// The display name of the role at `index`, in the same order [`OmarchyLibThemeSnapshot::roles`] uses.
///
/// A static string, owned by the library — never freed by the caller. Returns null for an index
/// past [`OMARCHY_LIB_THEME_ROLE_COUNT`].
#[unsafe(no_mangle)]
pub extern "C" fn omarchy_lib_theme_role_name(index: usize) -> *const c_char {
    let name: &CStr = match Role::ALL.get(index) {
        Some(Role::Background) => c"Background",
        Some(Role::Elevated) => c"Elevated",
        Some(Role::Recessed) => c"Recessed",
        Some(Role::Border) => c"Border",
        Some(Role::Primary) => c"Primary",
        Some(Role::Selection) => c"Selection",
        Some(Role::Muted) => c"Muted",
        Some(Role::Error) => c"Error",
        Some(Role::Focus) => c"Focus",
        Some(Role::Disabled) => c"Disabled",
        None => return ptr::null(),
    };
    name.as_ptr()
}

/// The display name of the treatment at `index`, in the same order [`treatment_at`]
/// reads. A static string, owned by the library — never freed by the caller. Returns null for an
/// index at or past [`OMARCHY_LIB_THEME_TREATMENT_COUNT`].
#[unsafe(no_mangle)]
pub extern "C" fn omarchy_lib_theme_treatment_name(index: usize) -> *const c_char {
    let name: &CStr = match Treatment::ALL.get(index) {
        Some(Treatment::Original) => c"Original",
        Some(Treatment::Inverted) => c"Inverted",
        Some(Treatment::HighContrast) => c"HighContrast",
        Some(Treatment::Mono) => c"Mono",
        Some(Treatment::Print) => c"Print",
        Some(Treatment::Sepia) => c"Sepia",
        None => return ptr::null(),
    };
    name.as_ptr()
}

/// The display name of the deposit at `index`, in the same order [`deposit_at`] reads. A static
/// string, owned by the library — never freed by the caller. Returns null for an index at or past
/// [`OMARCHY_LIB_THEME_DEPOSIT_COUNT`].
#[unsafe(no_mangle)]
pub extern "C" fn omarchy_lib_theme_deposit_name(index: usize) -> *const c_char {
    let name: &CStr = match Deposit::ALL.get(index) {
        Some(Deposit::Fill) => c"Fill",
        Some(Deposit::Outline) => c"Outline",
        Some(Deposit::Underline) => c"Underline",
        Some(Deposit::Highlight) => c"Highlight",
        Some(Deposit::Track) => c"Track",
        None => return ptr::null(),
    };
    name.as_ptr()
}

/// The display name of the state at `index`, in the same order [`state_at`] reads. A static
/// string, owned by the library — never freed by the caller. Returns null for an index at or past
/// [`OMARCHY_LIB_THEME_STATE_COUNT`].
#[unsafe(no_mangle)]
pub extern "C" fn omarchy_lib_theme_state_name(index: usize) -> *const c_char {
    let name: &CStr = match State::ALL.get(index) {
        Some(State::Rest) => c"Rest",
        Some(State::Hover) => c"Hover",
        Some(State::Pressed) => c"Pressed",
        Some(State::Disabled) => c"Disabled",
        Some(State::Focused) => c"Focused",
        None => return ptr::null(),
    };
    name.as_ptr()
}

/// Starts watching a theme: `path` a null-terminated path to a `colors.toml`, or null for the
/// system's active theme.
///
/// Returns null on failure, and — when `error_buffer` is non-null — writes a description into it,
/// truncated to fit `buffer_len` and always null-terminated when `buffer_len` is at least 1.
///
/// # Safety
///
/// `path`, if non-null, must point to a valid null-terminated C string. `error_buffer`, if
/// non-null, must be writable for `buffer_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omarchy_lib_theme_watch(
    path: *const c_char,
    error_buffer: *mut c_char,
    buffer_len: usize,
) -> *mut OmarchyLibThemeWatcher {
    let requested_path: Option<PathBuf> = if path.is_null() {
        None
    } else {
        // SAFETY: the caller promises `path` is a valid C string when non-null.
        let text = unsafe { CStr::from_ptr(path) }.to_string_lossy();
        Some(PathBuf::from(text.into_owned()))
    };
    let resolved = requested_path.unwrap_or_else(default_theme_path);

    match ThemeWatch::watch(&resolved) {
        Ok(watch) => Box::into_raw(Box::new(OmarchyLibThemeWatcher(watch))),
        Err(error) => {
            // SAFETY: the caller promises `error_buffer` is writable for `buffer_len` bytes when
            // non-null.
            unsafe { write_error(error_buffer, buffer_len, &error.to_string()) };
            ptr::null_mut()
        }
    }
}

/// Writes `message` into `buffer`, truncated to fit and null-terminated. A no-op when `buffer` is
/// null or `buffer_len` is zero.
///
/// # Safety
///
/// `buffer` must be writable for `buffer_len` bytes when non-null.
unsafe fn write_error(buffer: *mut c_char, buffer_len: usize, message: &str) {
    if buffer.is_null() || buffer_len == 0 {
        return;
    }

    let capacity = buffer_len - 1;
    let truncated = message
        .as_bytes()
        .get(..capacity)
        .unwrap_or(message.as_bytes());

    // SAFETY: the caller promises `buffer` is writable for `buffer_len` bytes; `truncated` fits in
    // `capacity` and leaves room for the trailing null the line after writes.
    unsafe {
        ptr::copy_nonoverlapping(truncated.as_ptr().cast(), buffer, truncated.len());
        *buffer.add(truncated.len()) = 0;
    }
}

/// The theme currently in force, taken through the treatment at `treatment` (see
/// [`omarchy_lib_theme_treatment_name`]; an out-of-range index falls back to `Original`).
///
/// # Safety
///
/// `watcher` must point to a live [`OmarchyLibThemeWatcher`], and `out` must be writable for one
/// [`OmarchyLibThemeSnapshot`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omarchy_lib_theme_current(
    watcher: *const OmarchyLibThemeWatcher,
    treatment: usize,
    out: *mut OmarchyLibThemeSnapshot,
) {
    // SAFETY: the caller promises both pointers are valid for this call.
    unsafe {
        let watch = &(*watcher).0;
        *out = snapshot(&watch.current().represented(treatment_at(treatment)));
    }
}

/// Writes into `*out` how the role at `role` sits on a piece, given the deposit at `deposit` and
/// the state at `state`, taken through the treatment at `treatment` (out-of-range indices fall back
/// to `Background`, `Fill`, `Rest`, and `Original`, respectively).
///
/// The result is an [`OmarchyLibThemeShade`]: `color` holds the deposit's `ground`, `content` its
/// `mark`. Resolved on demand, so it is not part of [`OmarchyLibThemeSnapshot`].
///
/// # Safety
///
/// `watcher` must point to a live [`OmarchyLibThemeWatcher`], and `out` must be writable for one
/// [`OmarchyLibThemeShade`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omarchy_lib_theme_deposit(
    watcher: *const OmarchyLibThemeWatcher,
    treatment: usize,
    role: usize,
    deposit: usize,
    state: usize,
    out: *mut OmarchyLibThemeShade,
) {
    // SAFETY: the caller promises both pointers are valid for this call.
    unsafe {
        let watch = &(*watcher).0;
        let styled = watch
            .current()
            .represented(treatment_at(treatment))
            .deposit(role_at(role), deposit_at(deposit), state_at(state));
        *out = OmarchyLibThemeShade {
            color: to_color(styled.ground),
            content: to_color(styled.mark),
        };
    }
}

/// Fills `out` and returns `true` when a new theme arrived since the last call, taken through the
/// treatment at `treatment`; leaves `out` untouched and returns `false` otherwise.
///
/// # Safety
///
/// `watcher` must point to a live [`OmarchyLibThemeWatcher`], and `out` must be writable for one
/// [`OmarchyLibThemeSnapshot`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omarchy_lib_theme_poll_changed(
    watcher: *const OmarchyLibThemeWatcher,
    treatment: usize,
    out: *mut OmarchyLibThemeSnapshot,
) -> bool {
    // SAFETY: the caller promises both pointers are valid for this call.
    unsafe {
        let watch = &(*watcher).0;
        match watch.changed() {
            Some(palette) => {
                *out = snapshot(&palette.represented(treatment_at(treatment)));
                true
            }
            None => false,
        }
    }
}

/// The file descriptor the caller integrates into its own event loop, readable whenever a valid
/// theme change landed. Reading from it never blocks; draining it fully is the caller's job.
///
/// # Safety
///
/// `watcher` must point to a live [`OmarchyLibThemeWatcher`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omarchy_lib_theme_signal_fd(
    watcher: *const OmarchyLibThemeWatcher,
) -> c_int {
    // SAFETY: the caller promises `watcher` is valid for this call.
    unsafe { (*watcher).0.signal_fd() }
}

/// Stops watching and releases `watcher`. A no-op on null; never call it twice on the same
/// pointer.
///
/// # Safety
///
/// `watcher` must be either null or a pointer this module returned from [`omarchy_lib_theme_watch`] that has
/// not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omarchy_lib_theme_free(watcher: *mut OmarchyLibThemeWatcher) {
    if watcher.is_null() {
        return;
    }
    // SAFETY: the caller promises `watcher` is a live pointer this module handed out.
    drop(unsafe { Box::from_raw(watcher) });
}

#[cfg(test)]
mod tests {
    use super::super::palette::PaletteSeed;
    use super::*;
    use crate::theme::read_theme;
    use std::ffi::CString;
    use std::path::Path;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    fn palette(name: &str) -> Palette {
        PaletteSeed::from_path(&fixture(name)).unwrap().extend()
    }

    #[test]
    fn carries_the_colors_the_theme_declares() {
        let theme = read_theme(&fixture("everforest.toml")).unwrap();
        let snapshot = snapshot(&palette("everforest.toml"));

        let red = snapshot.colors[0];
        assert_eq!(red.original.color, to_color(theme.colors.red));
        assert_eq!(red.dark.color, to_color(theme.colors.dark_red));
        assert_eq!(red.bright.color, to_color(theme.colors.bright_red));
        assert!(snapshot.colors.iter().all(|family| family.declared));
    }

    #[test]
    fn names_every_color_it_carries() {
        for index in 0..OMARCHY_LIB_THEME_COLOR_COUNT {
            assert!(!omarchy_lib_theme_color_name(index).is_null());
        }
        assert!(omarchy_lib_theme_color_name(OMARCHY_LIB_THEME_COLOR_COUNT).is_null());
    }

    /// The snapshot is pure marshalling: every role's color and content in it has to be exactly
    /// what `Palette` itself already returns for that role, at every level.
    #[test]
    fn matches_the_palette_it_was_built_from() {
        let palette = palette("everforest.toml");
        let built = snapshot(&palette);

        for (index, role) in SNAPSHOT_ROLES.into_iter().enumerate() {
            assert_eq!(built.roles[index].color, to_color(palette.color(role)));
            assert_eq!(
                built.roles[index].content_main,
                to_color(palette.content(role, Level::Primary))
            );
            assert_eq!(
                built.roles[index].content_secondary,
                to_color(palette.content(role, Level::Secondary))
            );
            assert_eq!(
                built.roles[index].content_disabled,
                to_color(palette.content(role, Level::Disabled))
            );
        }
    }

    #[test]
    fn carries_the_theme_mode() {
        assert!(snapshot(&palette("everforest.toml")).dark);
        assert!(!snapshot(&palette("flexoki-light.toml")).dark);
    }

    #[test]
    fn applies_the_requested_treatment() {
        let dark = palette("everforest.toml");

        // Inverted flips the mode; Original leaves it as declared.
        assert!(!snapshot(&dark.represented(Treatment::Inverted)).dark);
        assert!(snapshot(&dark.represented(Treatment::Original)).dark);
    }

    #[test]
    fn names_every_role_and_stops_at_the_end() {
        for (index, role) in Role::ALL.into_iter().enumerate() {
            let name = unsafe { CStr::from_ptr(omarchy_lib_theme_role_name(index)) }
                .to_str()
                .unwrap();
            assert_eq!(name, format!("{role:?}"));
        }

        assert!(omarchy_lib_theme_role_name(OMARCHY_LIB_THEME_ROLE_COUNT).is_null());
    }

    #[test]
    fn names_every_treatment_and_stops_at_the_end() {
        for (index, treatment) in Treatment::ALL.into_iter().enumerate() {
            let name = unsafe { CStr::from_ptr(omarchy_lib_theme_treatment_name(index)) }
                .to_str()
                .unwrap();
            assert_eq!(name, format!("{treatment:?}"));
        }

        assert!(omarchy_lib_theme_treatment_name(OMARCHY_LIB_THEME_TREATMENT_COUNT).is_null());
    }

    #[test]
    fn names_every_deposit_and_stops_at_the_end() {
        assert_eq!(OMARCHY_LIB_THEME_DEPOSIT_COUNT, Deposit::ALL.len());
        for (index, deposit) in Deposit::ALL.into_iter().enumerate() {
            let name = unsafe { CStr::from_ptr(omarchy_lib_theme_deposit_name(index)) }
                .to_str()
                .unwrap();
            assert_eq!(name, format!("{deposit:?}"));
        }

        assert!(omarchy_lib_theme_deposit_name(OMARCHY_LIB_THEME_DEPOSIT_COUNT).is_null());
    }

    #[test]
    fn names_every_state_and_stops_at_the_end() {
        assert_eq!(OMARCHY_LIB_THEME_STATE_COUNT, State::ALL.len());
        for (index, state) in State::ALL.into_iter().enumerate() {
            let name = unsafe { CStr::from_ptr(omarchy_lib_theme_state_name(index)) }
                .to_str()
                .unwrap();
            assert_eq!(name, format!("{state:?}"));
        }

        assert!(omarchy_lib_theme_state_name(OMARCHY_LIB_THEME_STATE_COUNT).is_null());
    }

    #[test]
    fn falls_back_to_original_for_an_out_of_range_treatment() {
        assert_eq!(
            treatment_at(OMARCHY_LIB_THEME_TREATMENT_COUNT),
            Treatment::Original
        );
    }

    #[test]
    fn falls_back_to_the_neutral_index_readers() {
        assert_eq!(deposit_at(OMARCHY_LIB_THEME_DEPOSIT_COUNT), Deposit::Fill);
        assert_eq!(state_at(OMARCHY_LIB_THEME_STATE_COUNT), State::Rest);
        assert_eq!(role_at(OMARCHY_LIB_THEME_ROLE_COUNT), Role::Background);
    }

    #[test]
    fn resolves_the_same_deposit_the_palette_does() {
        let path = CString::new(fixture("everforest.toml").to_str().unwrap()).unwrap();

        unsafe {
            let watcher = omarchy_lib_theme_watch(path.as_ptr(), ptr::null_mut(), 0);
            assert!(!watcher.is_null());

            let watch = &(*watcher).0;
            for (treatment_index, treatment) in [(0, Treatment::Original), (1, Treatment::Inverted)]
            {
                let palette = watch.current().represented(treatment);
                for (role_index, role) in SNAPSHOT_ROLES.into_iter().enumerate() {
                    for (deposit_index, deposit) in Deposit::ALL.into_iter().enumerate() {
                        for (state_index, state) in State::ALL.into_iter().enumerate() {
                            let mut out = std::mem::zeroed::<OmarchyLibThemeShade>();
                            omarchy_lib_theme_deposit(
                                watcher,
                                treatment_index,
                                role_index,
                                deposit_index,
                                state_index,
                                &mut out,
                            );

                            let expected = palette.deposit(role, deposit, state);
                            assert_eq!(out.color, to_color(expected.ground));
                            assert_eq!(out.content, to_color(expected.mark));
                        }
                    }
                }
            }

            omarchy_lib_theme_free(watcher);
        }
    }

    #[test]
    fn opens_reads_and_closes_a_watcher() {
        let path = CString::new(fixture("everforest.toml").to_str().unwrap()).unwrap();

        unsafe {
            let watcher = omarchy_lib_theme_watch(path.as_ptr(), ptr::null_mut(), 0);
            assert!(!watcher.is_null());

            let mut out = std::mem::zeroed::<OmarchyLibThemeSnapshot>();
            omarchy_lib_theme_current(watcher, 0, &mut out);
            assert!(out.dark);

            assert!(omarchy_lib_theme_signal_fd(watcher) >= 0);

            omarchy_lib_theme_free(watcher);
        }
    }

    #[test]
    fn reports_a_missing_theme_through_the_error_buffer() {
        let path = CString::new(fixture("does-not-exist.toml").to_str().unwrap()).unwrap();
        let mut buffer = [0 as c_char; 128];

        unsafe {
            let watcher = omarchy_lib_theme_watch(path.as_ptr(), buffer.as_mut_ptr(), buffer.len());
            assert!(watcher.is_null());

            let message = CStr::from_ptr(buffer.as_ptr()).to_str().unwrap();
            assert!(message.contains("no theme found"), "got: {message}");
        }
    }

    #[test]
    fn truncates_a_message_too_long_for_the_buffer() {
        let path = CString::new(fixture("does-not-exist.toml").to_str().unwrap()).unwrap();
        let mut buffer = [1 as c_char; 8];

        unsafe {
            omarchy_lib_theme_watch(path.as_ptr(), buffer.as_mut_ptr(), buffer.len());
            assert_eq!(buffer[7], 0, "the last byte must stay the null terminator");
        }
    }
}
