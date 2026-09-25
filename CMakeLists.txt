cmake_minimum_required(VERSION 3.20)

# The version a consumer links against — read from Cargo.toml so the two never drift, since
# Cargo.toml is the single source this project versions the C ABI from (see docs/BRIDGE.md §
# Versioning). A beta reads as three dot-separated numbers to CMake's project(VERSION); the
# pre-release suffix is stripped for that call and restated in full everywhere the version is
# actually published (the pkg-config file, the package config).
file(READ "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml" CARGO_TOML)
string(REGEX MATCH "\nversion = \"([^\"]+)\"" _ "${CARGO_TOML}")
set(OMARCHY_LIB_THEME_FULL_VERSION "${CMAKE_MATCH_1}")
string(REGEX MATCH "^[0-9]+\\.[0-9]+\\.[0-9]+" OMARCHY_LIB_THEME_NUMERIC_VERSION
       "${OMARCHY_LIB_THEME_FULL_VERSION}")

# The SONAME major this build produces — build.rs sets the same number on the .so it links, and the
# two have to move together by hand whenever the C ABI in src/api/ffi.rs breaks.
set(OMARCHY_LIB_THEME_ABI_MAJOR 1)

project(omarchy-lib-theme
    VERSION "${OMARCHY_LIB_THEME_NUMERIC_VERSION}"
    DESCRIPTION "Colors for Omarchy applications, inherited from the system active theme."
    LANGUAGES C
)

include(GNUInstallDirs)

set(RUST_PROFILE release)
set(RUST_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/src/target/${RUST_PROFILE}")
set(RUST_SHAREDLIB "${RUST_TARGET_DIR}/libomarchy_lib_theme.so")

add_custom_target(omarchy_lib_theme_rust ALL
    COMMAND cargo build --release
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    BYPRODUCTS "${RUST_SHAREDLIB}"
    COMMENT "Building the Rust color library (cdylib)"
    VERBATIM
)

# The .so is installed under its full version, with the two symlinks a consumer and the dynamic
# linker each expect: the SONAME symlink (libomarchy_lib_theme.so.1) resolves at load time, and the
# unversioned one (libomarchy_lib_theme.so) is what -lomarchy_lib_theme finds at link time.
set(SONAME_LINK "${CMAKE_CURRENT_BINARY_DIR}/libomarchy_lib_theme.so.${OMARCHY_LIB_THEME_ABI_MAJOR}")
set(DEV_LINK "${CMAKE_CURRENT_BINARY_DIR}/libomarchy_lib_theme.so")

add_custom_command(
    OUTPUT "${SONAME_LINK}" "${DEV_LINK}"
    COMMAND "${CMAKE_COMMAND}" -E create_symlink
            "libomarchy_lib_theme.so.${OMARCHY_LIB_THEME_FULL_VERSION}" "${SONAME_LINK}"
    COMMAND "${CMAKE_COMMAND}" -E create_symlink
            "libomarchy_lib_theme.so.${OMARCHY_LIB_THEME_ABI_MAJOR}" "${DEV_LINK}"
    DEPENDS omarchy_lib_theme_rust
    COMMENT "Linking the .so's SONAME and development symlinks"
)
add_custom_target(omarchy_lib_theme_links ALL DEPENDS "${SONAME_LINK}" "${DEV_LINK}")

install(FILES "${RUST_SHAREDLIB}"
    DESTINATION "${CMAKE_INSTALL_LIBDIR}"
    RENAME "libomarchy_lib_theme.so.${OMARCHY_LIB_THEME_FULL_VERSION}"
)
install(FILES "${SONAME_LINK}" "${DEV_LINK}"
    DESTINATION "${CMAKE_INSTALL_LIBDIR}"
)
install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/include/omarchy_lib_theme.h"
    DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}"
)
install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/LICENSE"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/licenses/omarchy-lib-theme"
)
install(FILES
    "${CMAKE_CURRENT_SOURCE_DIR}/docs/API.md"
    "${CMAKE_CURRENT_SOURCE_DIR}/docs/BRIDGE.md"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/doc/omarchy-lib-theme"
)

configure_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/omarchy-lib-theme.pc.in"
    "${CMAKE_CURRENT_BINARY_DIR}/omarchy-lib-theme.pc"
    @ONLY
)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/omarchy-lib-theme.pc"
    DESTINATION "${CMAKE_INSTALL_LIBDIR}/pkgconfig"
)

# No CMake target is built here — the two libraries are Cargo's output, only copied into place —
# so the package config below defines IMPORTED targets by hand instead of an install(EXPORT) set,
# which needs targets CMake itself built.
include(CMakePackageConfigHelpers)
configure_package_config_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/cmake/omarchy-lib-themeConfig.cmake.in"
    "${CMAKE_CURRENT_BINARY_DIR}/omarchy-lib-themeConfig.cmake"
    INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/omarchy-lib-theme"
    PATH_VARS CMAKE_INSTALL_LIBDIR CMAKE_INSTALL_INCLUDEDIR
)
write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/omarchy-lib-themeConfigVersion.cmake"
    VERSION "${OMARCHY_LIB_THEME_NUMERIC_VERSION}"
    COMPATIBILITY SameMajorVersion
)
install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/omarchy-lib-themeConfig.cmake"
    "${CMAKE_CURRENT_BINARY_DIR}/omarchy-lib-themeConfigVersion.cmake"
    DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/omarchy-lib-theme"
)
