/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// C FFI ABI version expected by downstream wrappers.
pub const ABI_VERSION: u32 = 1;

/// FFI supports ordered route-hop construction.
pub const CAP_ROUTE_PATH_ORDERED_HOPS: u64 = 0x0000_0000_0000_0001;

/// FFI supports the v1 native batch execution endpoint.
pub const CAP_BATCH_EXECUTE_V1: u64 = 0x0000_0000_0000_0002;

/// FFI supports diagnostics snapshot export as JSON.
pub const CAP_DIAGNOSTICS_JSON: u64 = 0x0000_0000_0000_0004;

/// FFI supports tag-group subscription endpoints.
pub const CAP_TAG_GROUP_SUBSCRIPTIONS: u64 = 0x0000_0000_0000_0008;

/// Capability bitmap for ABI v1.
pub const CAPABILITIES: u64 = CAP_ROUTE_PATH_ORDERED_HOPS
    | CAP_BATCH_EXECUTE_V1
    | CAP_DIAGNOSTICS_JSON
    | CAP_TAG_GROUP_SUBSCRIPTIONS;

/// Major version number
pub const MAJOR_VERSION: u32 = 1;

/// Minor version number
pub const MINOR_VERSION: u32 = 0;

/// Patch version number
pub const PATCH_VERSION: u32 = 0;

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
#[must_use]
pub fn get_version() -> &'static str {
    VERSION
}

/// Get the library name
#[must_use]
pub fn get_name() -> &'static str {
    NAME
}

/// Get the library description
#[must_use]
pub fn get_description() -> &'static str {
    DESCRIPTION
}
