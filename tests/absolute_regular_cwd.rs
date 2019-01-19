mod absolute_helpers;

fn setup() {
    #[cfg(windows)]
    std::env::set_current_dir(r"C:\").expect("Could not change to a regular directory");

    // For cfg(unix), we're always in a regular directory, so we don't need to
    // do anything special.
}
