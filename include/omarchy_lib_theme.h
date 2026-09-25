// The C ABI: a separate contract from the Rust API's v1 (docs/API.md), not an extension of it — see
// BRIDGE.md for the usage flow, the threading contract and what each treatment does.
//
// Hand-written to mirror src/api/ffi.rs exactly; nothing generates it, so a change on the Rust side
// has to be copied here by hand.
#pragma once

// This header is plain C, not C++: `typedef struct` and `#define` are its idiomatic, portable
// form, valid from a C compiler too. The NOLINT below silences the modernize/cppcoreguidelines
// clang-tidy checks a C++ consumer might run over it, which would otherwise push this file
// towards C++-only syntax it cannot use.
// NOLINTBEGIN(modernize-use-using,modernize-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays,cppcoreguidelines-macro-usage)

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// An RGB color, one byte per channel — the same precision colors.toml declares.
typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
} OmarchyLibThemeColor;

// One role: its own color, and the three weights of content that read on it.
typedef struct {
    OmarchyLibThemeColor color;
    OmarchyLibThemeColor content_main;
    OmarchyLibThemeColor content_secondary;
    OmarchyLibThemeColor content_disabled;
} OmarchyLibThemeRole;

// How many roles a snapshot carries.
#define OMARCHY_LIB_THEME_ROLE_COUNT 10

// One shade of a color: the shade itself and the content that reads on it.
typedef struct {
    OmarchyLibThemeColor color;
    OmarchyLibThemeColor content;
} OmarchyLibThemeShade;

// One color of the theme, as the family of three it reads as: the color itself, its dark shade and
// its bright one, in that order.
typedef struct {
    OmarchyLibThemeShade original;
    OmarchyLibThemeShade dark;
    OmarchyLibThemeShade bright;
    bool declared;
} OmarchyLibThemeColorFamily;

// How many colors a snapshot carries, each as a family of three shades.
#define OMARCHY_LIB_THEME_COLOR_COUNT 6

// Every role resolved against the theme in force, in the fixed order omarchy_lib_theme_role_name names, and
// every color the theme declares, in the order omarchy_lib_theme_color_name names.
typedef struct {
    OmarchyLibThemeRole roles[OMARCHY_LIB_THEME_ROLE_COUNT];
    OmarchyLibThemeColorFamily colors[OMARCHY_LIB_THEME_COLOR_COUNT];
    bool dark;
} OmarchyLibThemeSnapshot;

// An opaque handle to a running watcher. Only ever seen through a pointer.
typedef struct OmarchyLibThemeWatcher OmarchyLibThemeWatcher;

// The display name of the role at `index`, in the same order OmarchyLibThemeSnapshot::roles uses.
//
// A static string, owned by the library — never freed by the caller. Returns NULL for an index at
// or past OMARCHY_LIB_THEME_ROLE_COUNT.
const char *omarchy_lib_theme_role_name(size_t index);

// The display name of the color at `index`, in the same order OmarchyLibThemeSnapshot::colors uses.
//
// A static string, owned by the library — never freed by the caller. Returns NULL for an index at
// or past OMARCHY_LIB_THEME_COLOR_COUNT.
const char *omarchy_lib_theme_color_name(size_t index);

// How many treatments omarchy_lib_theme_treatment_name can name.
#define OMARCHY_LIB_THEME_TREATMENT_COUNT 6

// The display name of the treatment at `index` — Original, Inverted, HighContrast, Mono,
// Print or Sepia, in that order.
//
// A static string, owned by the library — never freed by the caller. Returns NULL for an index at
// or past OMARCHY_LIB_THEME_TREATMENT_COUNT.
const char *omarchy_lib_theme_treatment_name(size_t index);

// How many deposits omarchy_lib_theme_deposit_name can name.
#define OMARCHY_LIB_THEME_DEPOSIT_COUNT 5

// The display name of the deposit at `index` — Fill, Outline, Underline, Highlight or Track, in
// that order.
//
// A static string, owned by the library — never freed by the caller. Returns NULL for an index at
// or past OMARCHY_LIB_THEME_DEPOSIT_COUNT.
const char *omarchy_lib_theme_deposit_name(size_t index);

// How many states omarchy_lib_theme_state_name can name.
#define OMARCHY_LIB_THEME_STATE_COUNT 5

// The display name of the state at `index` — Rest, Hover, Pressed, Disabled or Focused, in that
// order.
//
// A static string, owned by the library — never freed by the caller. Returns NULL for an index at
// or past OMARCHY_LIB_THEME_STATE_COUNT.
const char *omarchy_lib_theme_state_name(size_t index);

// Starts watching a theme: `path` a null-terminated path to a colors.toml, or NULL for the
// system's active theme.
//
// Returns NULL on failure, and — when `error_buffer` is non-NULL — writes a description into it,
// truncated to fit `buffer_len` and always null-terminated when `buffer_len` is at least 1.
OmarchyLibThemeWatcher *omarchy_lib_theme_watch(const char *path, char *error_buffer, size_t buffer_len);

// Writes the theme currently in force into `*out`, taken through the treatment at
// `treatment` (an out-of-range index falls back to Original).
void omarchy_lib_theme_current(const OmarchyLibThemeWatcher *watcher, size_t treatment, OmarchyLibThemeSnapshot *out);

// Writes into `*out` how the role at `role` sits on a piece, given the deposit at `deposit` and
// the state at `state`, taken through the treatment at `treatment` (out-of-range indices fall back
// to Background, Fill, Rest and Original). `color` holds the deposit's ground, `content` its mark.
//
// Resolved on demand, so it is not part of OmarchyLibThemeSnapshot. A role index reads the same
// fixed order omarchy_lib_theme_role_name names.
void omarchy_lib_theme_deposit(const OmarchyLibThemeWatcher *watcher, size_t treatment, size_t role, size_t deposit, size_t state, OmarchyLibThemeShade *out);

// Fills `*out` and returns true when a new theme arrived since the last call, taken through the
// treatment at `treatment`; leaves `*out` untouched and returns false otherwise.
bool omarchy_lib_theme_poll_changed(const OmarchyLibThemeWatcher *watcher, size_t treatment, OmarchyLibThemeSnapshot *out);

// A file descriptor the caller integrates into its own event loop (e.g. QSocketNotifier),
// readable whenever a valid theme change landed. Reading from it never blocks; draining it fully
// before the next wait is the caller's job.
int omarchy_lib_theme_signal_fd(const OmarchyLibThemeWatcher *watcher);

// Stops watching and releases `watcher`. A no-op on NULL; never call it twice on the same pointer.
void omarchy_lib_theme_free(OmarchyLibThemeWatcher *watcher);

#ifdef __cplusplus
}
#endif

// NOLINTEND(modernize-use-using,modernize-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays,cppcoreguidelines-macro-usage)
