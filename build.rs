fn main() {
    // Emit build/git metadata for `src/version.rs`. Building from a crates.io
    // tarball has no `.git` directory, so a git failure here must NOT fail the
    // build — `src/version.rs` reads these vars with `option_env!` and falls
    // back to "unknown" when they are absent.
    if let Err(e) = vergen::EmitBuilder::builder()
        .build_timestamp()
        .git_sha(true)
        .git_branch()
        .git_commit_timestamp()
        .emit()
    {
        println!("cargo:warning=vergen build/git metadata unavailable: {e}");
    }
}
