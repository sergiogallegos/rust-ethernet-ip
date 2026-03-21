# Contributing to rust-ethernet-ip

Thanks for contributing. This project targets production-grade EtherNet/IP communication for Allen-Bradley PLCs, so correctness and regression safety are prioritized over speed of merge.

## Scope and Release Line

- `0.6.3` is the latest published stable crate line.
- Current work on `main` is preparing `0.7.0` (unreleased).
- Do not bump crate/package version unless explicitly requested during release cut.

## Development Workflow

1. Fork and create a branch from `main`.
2. Keep changes focused (one concern per PR when possible).
3. Add or update tests for behavior changes.
4. Run required checks locally.
5. Update docs/changelog if user-facing behavior changed.
6. Open PR with clear risk notes and test evidence.

## Required Checks

Run these before opening a PR:

```bash
cargo fmt
cargo clippy -p rust-ethernet-ip --lib -- -D warnings
cargo test --workspace --all-targets
dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj -v minimal
```

For PLC-dependent validation, include what was tested on real hardware (model, firmware, route path, tags used).

## Testing Guidelines

- Add regression tests for every bug fix.
- Prefer deterministic simulator tests when possible.
- Keep FFI boundary tests strict on return codes and payload shape.
- For cross-language changes (Rust <-> C#), include tests on both sides.

## Coding Standards

- Keep public APIs and behavior backwards-compatible unless change is intentional and documented.
- Use clear error messages and typed errors where possible.
- Avoid unrelated refactors in bug-fix PRs.
- Update `CHANGELOG.md` under `Unreleased` for notable changes.

## Pull Request Checklist

- [ ] Tests added/updated for changed behavior
- [ ] Rust checks pass
- [ ] C# tests pass
- [ ] Documentation updated (README/docs/changelog) if needed
- [ ] Breaking changes called out explicitly

## Bug Reports

When filing issues, include:

- PLC model and firmware
- Network/routing details (direct vs backplane/slot)
- Exact tag path(s)
- Minimal repro code
- Expected vs actual result
- Logs/error codes
