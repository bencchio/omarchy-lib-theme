# The C bridge

`omarchy_lib_theme.h` is a separate contract from the Rust API `API.md`
promotes — it is *not* an extension of v1. v1 is palettes only: a basic
palette in, an extended legible one out. The C bridge exists because a v1
palette is a snapshot, and a real application needs to follow the active
theme as it changes and see it through more than one treatment — both
of which sit behind types v1 explicitly keeps out of its own surface
(`ThemeWatcher`, `Treatment`). The bridge is where those two live, in a
form any language with C FFI can call.

## Ownership

Four rules cover every pointer that crosses the bridge, so no function needs to
be reread to confirm the pattern:

| What | Who owns it | What the caller does |
| --- | --- | --- |
| The strings from `omarchy_lib_theme_role_name`, `omarchy_lib_theme_color_name`, `omarchy_lib_theme_treatment_name`, `omarchy_lib_theme_deposit_name`, `omarchy_lib_theme_state_name` | The library | Reads them; never frees them. They are static and outlive any watcher. An out-of-range index gives `NULL`. |
| The `OmarchyLibThemeWatcher *` from `omarchy_lib_theme_watch` | The caller | Frees it exactly once, with `omarchy_lib_theme_free`. Never twice, never while another call on it is running. |
| The `OmarchyLibThemeSnapshot` passed to `omarchy_lib_theme_current` and `omarchy_lib_theme_poll_changed`, and the `OmarchyLibThemeShade` passed to `omarchy_lib_theme_deposit` | The caller | Provides the storage; the library fills it by copy. It holds no pointers, so it stays valid after the watcher is freed. |
| The `error_buffer` passed to `omarchy_lib_theme_watch` | The caller | Provides it and its size; the library writes a null-terminated message into it only on failure. Passing `NULL` is allowed. |

The file descriptor from `omarchy_lib_theme_signal_fd` belongs to the watcher: read from
it, never close it, and stop using it once the watcher is freed.

## Usage flow

```c
#include "omarchy_lib_theme.h"
#include <stdio.h>
#include <poll.h>
#include <unistd.h>

int main(void) {
    char error[256];
    OmarchyLibThemeWatcher *watcher = omarchy_lib_theme_watch(NULL, error, sizeof(error));
    if (watcher == NULL) {
        fprintf(stderr, "could not watch the theme: %s\n", error);
        return 1;
    }

    OmarchyLibThemeSnapshot snapshot;
    omarchy_lib_theme_current(watcher, 0, &snapshot); /* 0 = Original */
    printf("starting on %s theme\n", snapshot.dark ? "a dark" : "a light");

    struct pollfd watched = { .fd = omarchy_lib_theme_signal_fd(watcher), .events = POLLIN };
    while (poll(&watched, 1, -1) > 0) {
        if (omarchy_lib_theme_poll_changed(watcher, 0, &snapshot)) {
            printf("theme changed — now %s\n", snapshot.dark ? "dark" : "light");
        }
    }

    omarchy_lib_theme_free(watcher);
    return 0;
}
```

`omarchy_lib_theme_signal_fd` gives a file descriptor readable whenever a valid change
landed — integrate it into whatever event loop the application already runs
(`poll`, `select`, Qt's `QSocketNotifier`, a Go `epoll` wrapper, …) instead of
polling on a timer. Reading it never blocks; draining it fully before the
next wait is the caller's job, same as any other readiness fd.

### In a Qt event loop

The same flow with no `poll()` of its own, because the application already has
an event loop. A `ThemeBridge` `QObject` wraps the watcher and lets QML read the
snapshot; the notification fd goes into a `QSocketNotifier`.

```cpp
ThemeBridge::ThemeBridge(QObject *parent) : QObject(parent) {
    std::array<char, 256> error{};
    m_watcher = omarchy_lib_theme_watch(nullptr, error.data(), error.size());
    if (m_watcher == nullptr) {
        m_startupError = QString::fromUtf8(error.data());
        return;
    }

    refresh();  // draw once on what the theme is right now

    m_notifier = new QSocketNotifier(omarchy_lib_theme_signal_fd(m_watcher),
                                     QSocketNotifier::Read, this);
    connect(m_notifier, &QSocketNotifier::activated,
            this, &ThemeBridge::onSignalReadable);
}

ThemeBridge::~ThemeBridge() {
    omarchy_lib_theme_free(m_watcher);
}

void ThemeBridge::onSignalReadable() {
    // Drain the fd before polling: several changes can coalesce between two
    // wakeups, and only the newest one still matters.
    std::array<char, 64> discard{};
    const auto fd = static_cast<int>(m_notifier->socket());
    while (read(fd, discard.data(), discard.size()) > 0) {
    }

    OmarchyLibThemeSnapshot snapshot{};
    if (omarchy_lib_theme_poll_changed(m_watcher, m_treatment, &snapshot)) {
        applySnapshot(snapshot);  // redraw
    }
}
```

The shape is the same in any event loop: register the fd, drain it when it
wakes, ask the watcher whether anything actually changed, redraw only if it
did.

`omarchy_lib_theme_current` and `omarchy_lib_theme_poll_changed` both take a treatment index
— see below — and write a full `OmarchyLibThemeSnapshot` either way; the difference
is that `omarchy_lib_theme_poll_changed` returns `false` and leaves `*out` untouched
when nothing changed since the last call.

## Thread safety

`omarchy_lib_theme_current` and `omarchy_lib_theme_poll_changed` may be called from any thread
for as long as the `OmarchyLibThemeWatcher` they take is alive. The state behind a
watcher sits behind a mutex; the thread that watches the filesystem for
changes updates it independently of whatever thread is reading. A read never
blocks waiting on that thread, and never returns a torn snapshot.

`omarchy_lib_theme_free` is the one call that isn't safe to race: never call it
concurrently with another call on the same watcher, and never call it twice
on the same pointer.

## Treatments

Every function that takes a `size_t treatment` reads it against this
fixed order — the same `omarchy_lib_theme_treatment_name` names:

| Index | Name | What it does |
| --- | --- | --- |
| 0 | Original | The theme exactly as declared. |
| 1 | Inverted | The same identity with its polarity flipped — dark becomes light or light becomes dark, and each color keeps its own hue, only restated for the ground it now sits on. |
| 2 | HighContrast | Every color pushed further from the background than it already stood, for readability. |
| 3 | Mono | The theme's own scale, background to foreground, with no hue. |
| 4 | Print | Forced onto a white page: pure white behind, pure black text, every other color reduced to the grey its own weight gives it. |
| 5 | Sepia | Forced onto aged paper: a warm ground behind, dark brown text, and every other color a shade of that one brown, told apart by the tone its own weight gives it. |

An out-of-range index falls back to Original rather than failing — there is
no error path for it.

A treatment applies on top of the mode a theme already resolves to
(`OmarchyLibThemeSnapshot.dark`) — it is not a peer of that flag, it is an
operation that runs before the flag is read. Three of the six change the
resulting mode by design: Inverted restates the same theme for the opposite
ground, which is its whole point, not a side effect; Print and Sepia always
resolve to a light snapshot, because the page they target is light, unrelated
to polarity. Original, HighContrast, and Mono leave the mode untouched.

## Deposits and states

The snapshot says which color each role resolves to. It does not say how that
color sits on a piece of interface, or what situation the piece is in — those
are two small vocabularies an interface combines with a role. A *deposit* is
the way a color is laid down; a *state* is the situation the piece is in.
Neither names a widget, so both cover components that do not exist yet.

A deposit is read against this fixed order — the same
`omarchy_lib_theme_deposit_name` names:

| Index | Name | What `color` holds | What `content` holds |
| --- | --- | --- | --- |
| 0 | Fill | The fill. | The content that reads on it. |
| 1 | Outline | The line. | The content that reads on the canvas behind it. |
| 2 | Underline | The line. | The content that reads on the canvas behind it. |
| 3 | Highlight | The fill. | The content that reads on it. |
| 4 | Track | The track. | The indicator that travels it. |

`Highlight` and `Fill` resolve alike today; they stay separate so a caller can
say which it means and the two can diverge later.

A state is read against this fixed order — the same
`omarchy_lib_theme_state_name` names:

| Index | Name | What it does |
| --- | --- | --- |
| 0 | Rest | The piece as it sits. |
| 1 | Hover | Moves the role's color away from the canvas. |
| 2 | Pressed | Moves it toward the canvas. |
| 3 | Disabled | Moves it toward the canvas and drops its chroma. |
| 4 | Focused | Moves it away from the canvas, further than hover. |

One call resolves a triple, on demand — not from a value baked into the
snapshot:

```c
OmarchyLibThemeShade shade;
omarchy_lib_theme_deposit(watcher, treatment, role, deposit, state, &shade);
/* role 4 = Primary, deposit 0 = Fill, state 1 = Hover, treatment 0 = Original */
```

It takes a treatment index exactly like `omarchy_lib_theme_current`, so a
consumer that switches treatment applies that treatment to everything it draws.
`color` and `content` hold the pair the deposit names in the table above. An
out-of-range index falls back instead of failing: `role` to Background, `deposit`
to Fill, `state` to Rest, and `treatment` to Original — the same no-error-path
contract the other index readers keep. Because it resolves on demand, it adds
nothing to `OmarchyLibThemeSnapshot` and does not change its layout.

## Versioning

The shared object's SONAME (`libomarchy_lib_theme.so.1`) names the ABI
generation: the layout of `OmarchyLibThemeColor`, `OmarchyLibThemeRole`, `OmarchyLibThemeShade`,
`OmarchyLibThemeColorFamily`, `OmarchyLibThemeSnapshot`, and the signature of every
`omarchy_*` function. Past `1.0.0` it moves whenever one of those breaks.

Before `1.0.0` it does not. The package version (`Cargo.toml`, the `.pc`
file, the CMake package) is `1.0.0-beta.N` right now, and a beta is the
generation still being shaped: a layout may change from one beta to the next
— a role added to `OmarchyLibThemeSnapshot`, a field placed differently — without the
SONAME moving off `.1`. Recompile against the header of the beta you link.

The package version is what a consumer pins during the beta, and every such
change advances it. The bridge earns `1.0.0` without a suffix once an
application outside this repository has tried it; from there the SONAME
carries the promise, and a break moves it to `.2`.
