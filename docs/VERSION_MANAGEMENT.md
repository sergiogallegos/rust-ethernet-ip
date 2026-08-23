# Version Management

This document describes the version management process for the Rust EtherNet/IP library.

## Version Scheme

This project follows [Semantic Versioning](https://semver.org/) (SemVer):

- **MAJOR.MINOR.PATCH** (e.g., 1.0.0)
- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality in a backwards compatible manner
- **PATCH**: Backwards compatible bug fixes

## Current Version State

- **Current stable published version:** `1.2.1`
- **Previous stable version:** `1.2.0`
- **Earlier stable versions:** `1.0.0`, `0.7.0`, `0.6.3`

### Version History

| Version | Release Date | Status | Notes |
|---------|-------------|--------|-------|
| 1.2.1   | 2026-08-22  | Current published stable | Schema-cache safety and explicit refresh, native Python batch writes, cross-binding diagnostics, and controller-compatible tag-attribute fallback |
| 1.2.0   | 2026-07-10  | Previous published stable | Handle-aware and fragmented STRING/structure operations, packet-size-aware batches, C/C++ consumer support, diagnostics and lifecycle hardening |
| 1.1.0   | 2026-06-19  | Earlier stable | Cross-wrapper correctness, packaging, error propagation, and release automation |
| 1.0.0   | 2026-05-24  | Earlier stable | SemVer-major release-window bundle: actor refactor, sibling crates, FFI ABI handshake, BOOL array fixes, CIP path validation, Python typed writes, fleet pool, service layer, retry primitive |
| 0.7.0   | 2026-04-07  | Previous published stable | Hardening release with Rust/C# parity improvements and real-PLC validation evidence |
| 0.6.3   | 2026-03-01  | Previous stable | Reliability and protocol correctness fixes |
| 0.6.2   | 2026-01-24  | Previous stable | Stream injection and test configuration support |
| 0.6.1   | 2026-01-17  | Legacy stable | Repository scope cleanup |

## Files That Contain Version Information

The following files contain version information and must be updated when releasing a new version:

### Core Rust Files
- `Cargo.toml` - Main package version
- `crates/*/Cargo.toml` - Sibling crate package versions
- `src/version.rs` - Version constants
- `VERSION` - Simple version file

### C# Project Files
- `csharp/RustEtherNetIp/RustEtherNetIp.csproj` - Main C# library
- `examples/WpfExample/WpfExample.csproj` - WPF example
- `examples/WinFormsExample/WinFormsExample.csproj` - WinForms example
- `examples/AspNetExample/AspNetExample.csproj` - ASP.NET example

### Documentation Files
- `CHANGELOG.md` - Release notes and version history
- `README.md` - Quick start dependency version
- `docs/README.md`, `docs/programmer_manual.md`, `docs/SOFTWARE_ARCHITECTURE.md`, and wrapper README files - Current release-state references

## Automated Version Management

### Release Readiness Check

Before a release-prep commit or tag, run the mechanical readiness check:

```bash
scripts/check-release-readiness X.Y.Z
```

The script reads `scripts/check-release-readiness.txt`, verifies every known version-string site against `X.Y.Z`, runs Cargo package dry-runs in workspace publish order, and inspects each generated crate README for stale release or install versions. CI and the release workflow also run `scripts/check-packaged-release-docs` against generated NuGet and Python artifacts before publication. Use `--strict` for release-day validation after sibling crates have propagated on crates.io. Use `--ignore-examples` only when demo application versions intentionally diverge from the library version.

### Using the Update Script

Use the PowerShell script to automatically update versions across all files:

```powershell
# Update version only
.\scripts\update-version.ps1 -Version "0.3.0"

# Update version and add changelog entry
.\scripts\update-version.ps1 -Version "0.3.0" -UpdateChangelog
```

### Manual Version Update Process

If you prefer to update manually:

1. **Update Cargo.toml**
   ```toml
   version = "0.3.0"
   ```

2. **Update src/version.rs**
   ```rust
   pub const MAJOR_VERSION: u32 = 0;
   pub const MINOR_VERSION: u32 = 3;
   pub const PATCH_VERSION: u32 = 0;
   ```

3. **Update VERSION file**
   ```
   0.3.0
   ```

4. **Update C# project files**
   ```xml
   <Version>0.3.0</Version>
   <AssemblyVersion>0.3.0.0</AssemblyVersion>
   <FileVersion>0.3.0.0</FileVersion>
   ```

5. **Update CHANGELOG.md**
   Add new version entry with release date and changes.

## Release Process

### 1. Pre-Release Checklist

- [ ] `scripts/check-release-readiness X.Y.Z` passes
- [ ] All tests pass
- [ ] Documentation is updated
- [ ] Performance benchmarks are current
- [ ] Examples work with new version
- [ ] CHANGELOG.md is updated with all changes

### 2. Version Update

```powershell
# Update version across all files
.\scripts\update-version.ps1 -Version "X.Y.Z" -UpdateChangelog

# Review and edit CHANGELOG.md with actual changes
# Edit the generated changelog entry to include real changes
```

### 3. Build and Test

```bash
# Build Rust library
cargo build --release

# Test Rust library
cargo test

# Build C# examples
dotnet build csharp/RustEtherNetIp/RustEtherNetIp.csproj
dotnet build examples/WpfExample/WpfExample.csproj
dotnet build examples/WinFormsExample/WinFormsExample.csproj
dotnet build examples/AspNetExample/AspNetExample.csproj

# Run integration tests
cargo test --test integration_tests
```

### 4. Commit and Tag

```bash
# Commit version changes
git add .
git commit -m "Release version X.Y.Z"

# Create and push tag
git tag vX.Y.Z
git push origin main
git push origin vX.Y.Z
```

### 5. Publish (when ready)

```bash
# Publish to crates.io
cargo publish

# Publish Windows-first C# package to NuGet
cargo build --release
pwsh ./scripts/pack-nuget.ps1 -OutputDir ./artifacts/nuget
dotnet nuget push ./artifacts/nuget/RustEtherNetIp.X.Y.Z.nupkg --api-key "$NUGET_API_KEY" --source https://api.nuget.org/v3/index.json
```

## Version Planning

### Upcoming Versions

#### v0.3.0 (Q2 2025)
- Program Scope Tags support
- Real-time subscriptions
- Advanced connection pooling
- ControlLogix support

#### v0.4.0 (Q3 2025)
- Security features
- Advanced diagnostics
- Cloud integration capabilities

#### v0.5.0 (Q4 2025)
- Advanced analytics
- Multi-PLC coordination
- Production-ready release

## Version Compatibility

### Rust Library
- **0.2.x**: Compatible with Rust 1.70+
- **0.1.x**: Compatible with Rust 1.70+

### C# Bindings
- **0.2.x**: Compatible with .NET 6.0+, .NET 9.0+
- **0.1.x**: Compatible with .NET 6.0+

### PLC Compatibility
- **All versions**: CompactLogix L1x-L5x series
- **0.3.0+**: ControlLogix L6x-L7x series (planned)

## Breaking Changes Policy

### Major Version (X.0.0)
- API breaking changes allowed
- Migration guide provided
- Deprecation warnings in previous minor versions

### Minor Version (0.X.0)
- New features only
- Backwards compatible
- May deprecate features (with warnings)

### Patch Version (0.0.X)
- Bug fixes only
- Backwards compatible
- No new features

## Support Policy

- **Current version**: Full support
- **Previous minor version**: Security fixes only
- **Older versions**: Community support only

## Troubleshooting Version Issues

### Common Issues

1. **Version mismatch between Rust and C#**
   - Ensure all project files have the same version
   - Rebuild all projects after version update

2. **Git tag conflicts**
   - Check existing tags: `git tag -l`
   - Delete conflicting tag: `git tag -d vX.Y.Z`

3. **Build failures after version update**
   - Clean build: `cargo clean && dotnet clean`
   - Rebuild: `cargo build --release && dotnet build`

### Verification Commands

```bash
# Check current version in all files
grep -r "0\.2\.0" . --include="*.toml" --include="*.csproj" --include="*.rs" --include="*.md"

# Verify git tags
git tag -l | grep "v0"

# Check build status
cargo check && dotnet build --no-restore
``` 
