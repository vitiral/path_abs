//! This file tests PathAbs::new() for Windows when the current directory
//! uses extended-length path syntax (like `\\?\C:\`).

// These tests are already run for Unix in absolute_regular_cwd.rs, and Unix
// doesn't have "extended-length path syntax", so we can make them Windows-only
// here.
#[cfg(windows)]
mod absolute_helpers;

#[cfg(windows)]
fn setup() {
    std::env::set_current_dir(r"\\?\C:\").expect("Could not change to a regular directory");
}
