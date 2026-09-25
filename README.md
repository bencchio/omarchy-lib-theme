# omarchy-lib-theme

The theming library for **Omarchy** applications. It takes the active theme as
the source of truth and hands an application everything it needs to look like
it belongs: the colors as the roles an interface plays, the same theme through
six treatments, and the change notification that keeps all of it live.

An application that wants to fit into Omarchy does not build its own theming
system, generate shades, work out readable combinations or watch the theme for
changes. It gets all of that from one coherent source.

**The principle:** the Omarchy theme defines the visual identity; the library
expands it and makes it consumable by applications.

Install it and link it as `omarchy-lib-theme` — the shorter name it carries in
`pkg-config`, in CMake and in the shared object.

## Which contract do I need?

The library ships two separate contracts. They are not layers of one another —
pick by what the application has to do.

| If the application needs to… | Use | Documented in |
| --- | --- | --- |
| Resolve a palette once, from Rust — roles, readable content, tonal ramps | The v1 Rust API | [`docs/API.md`](docs/API.md) |
| Follow the active theme as it changes, or read it through more than one treatment, from any language with a C FFI | The C bridge | [`docs/BRIDGE.md`](docs/BRIDGE.md) |

A palette is a snapshot, which is why following the theme and the alternate
treatments live on the bridge instead: v1 is palettes only.

## Installing

### Requirements

- Rust with `cargo` — the library itself is Rust, and CMake invokes it.
- CMake 3.20 or newer.

### On Arch Linux

`packaging/arch/PKGBUILD` builds and installs the library with `makepkg`:

```sh
cd packaging/arch
makepkg -si
```

### Anywhere else

`install.sh` builds and installs the library without going through a
distribution package — the quick way to try a build before packaging it, or
on a system with no AUR:

```sh
./install.sh --prefix "$HOME/.local"
```

### From a release

Each release carries the Arch package already built for `x86_64`, so installing
it needs no toolchain:

<https://github.com/bencchio/omarchy-lib-theme/releases>

```sh
sudo pacman -U omarchy-lib-theme-<version>-1-x86_64.pkg.tar.zst
```

Releases also carry the source archives GitHub generates for the tag, for
building on anything that is not Arch.

### Build and install manually

For full control over the build, run the same steps `install.sh` wraps:

```sh
cmake -S . -B build -DCMAKE_INSTALL_PREFIX="$HOME/.local"
cmake --build build
cmake --install build
```

That installs the shared object under its full version with the two symlinks a
consumer expects, the C bridge header, the `pkg-config` file, the CMake
package, the license, and both contract documents.

The package version is `1.0.0-beta.N` and the SONAME is
`libomarchy_lib_theme.so.1`. Within a beta the snapshot layout may change without
the SONAME moving, so pin the package version and recompile against the header
of the beta you link — see `docs/BRIDGE.md` § Versioning.

### Use it from an application

With `pkg-config`:

```sh
pkg-config --cflags --libs omarchy-lib-theme
```

With CMake:

```cmake
find_package(omarchy-lib-theme REQUIRED)
target_link_libraries(my-app PRIVATE omarchy-lib-theme::shared)
```

If the prefix is not one the tools already search, point them at it —
`PKG_CONFIG_PATH` for the first, `CMAKE_PREFIX_PATH` for the second.

### See it working

The reference application lives in its own repository and builds against this
one's installed package:

<https://github.com/bencchio/omarchy-lib-theme-showcase>

## What it includes

- **Semantic colors** — the interface recommended for applications. The app
  states what part a color plays (`background`, `elevated`, `recessed`,
  `primary`, …) instead of knowing how it was produced.
- **Deposits and states** — how that color sits on a piece (`fill`, `outline`,
  `underline`, `highlight`, `track`) and in what situation the piece is
  (`rest`, `hover`, `pressed`, `disabled`, `focused`). The library resolves
  the pair; it still does not draw the piece.
- **Tonal ramps** — the tonal variants of the base colors, for cases that need
  finer control.
- **Treatments** — the same theme readable in several ways: Original,
  Inverted, High Contrast, Mono, Print and Sepia. They express visual intent, not
  arithmetic on RGB.
- **Live updates** — when the Omarchy theme changes, applications receive the
  change and redraw without watching files, without polling and without
  restarting.

## Stack

- **Library:** Rust.

## Outside this scope

Advanced accessibility — explicit contrast-level compliance, modes for the
different color vision deficiencies, automatic validation of combinations — is
later work. The library does avoid producing obviously problematic color
combinations.
