/// Get the version string for rex and librex
pub fn get_version_string() -> String {
    format!(
        "rex {}\nlibrex {}",
        env!("CARGO_PKG_VERSION"),
        librex::version()
    )
}

/// Print version information to stdout
pub fn print_version() {
    println!("{}", get_version_string());
}

#[cfg(test)]
#[path = "version_tests.rs"]
mod tests;
