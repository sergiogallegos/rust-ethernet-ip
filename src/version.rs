/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Major version number
pub const MAJOR_VERSION: u32 = 0;

/// Minor version number
pub const MINOR_VERSION: u32 = 5;

/// Patch version number
pub const PATCH_VERSION: u32 = 5;

/// Version string in format "v0.1.0"
pub const VERSION_STRING: &str = concat!("v", env!("CARGO_PKG_VERSION"));

/// Build date and time
pub const BUILD_DATE: &str = env!("VERGEN_BUILD_TIMESTAMP");

/// Git commit hash
pub const GIT_HASH: &str = env!("VERGEN_GIT_SHA");

/// Git commit date
pub const GIT_COMMIT_DATE: &str = env!("VERGEN_GIT_COMMIT_TIMESTAMP");

/// Git branch
pub const GIT_BRANCH: &str = env!("VERGEN_GIT_BRANCH");

/// Get the library version as a string
pub fn get_version() -> &'static str {
    VERSION
}

/// Get the library name
pub fn get_name() -> &'static str {
    NAME
}

/// Get the library description
pub fn get_description() -> &'static str {
    DESCRIPTION
}
