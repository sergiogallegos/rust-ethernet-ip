---
id: CODEX-AF
title: Full-coverage exerciser — cwd-independent manifest resolution across all three bindings
owner: codex
status: submitted
created: 2026-05-25
last-update: 2026-05-25 codex [gpt-5]
---

## Brief

### Goal

The CODEX-AE-unified hardware exercisers (`examples/test_plc_full_coverage.rs`, `examples/CSharpFullCoverage/Program.cs`, `python/examples/test_plc_full_coverage.py`) all resolve `examples/full_coverage_tags.json` relative to the current working directory. They only work when invoked from the repo root. Caught during the 2026-05-25 CompactLogix L18ER cross-binding validation:

```
$ cd examples/CSharpFullCoverage && dotnet run -c Release
Unhandled exception. System.IO.DirectoryNotFoundException:
  Could not find a part of the path
  '/Users/.../examples/CSharpFullCoverage/examples/full_coverage_tags.json'.
```

Workaround today is "always invoke from repo root" (e.g. `dotnet run --project examples/CSharpFullCoverage`), but that's an ergonomic gotcha that's easy to forget and not enforced anywhere. Fix the manifest resolution so each runner works from any cwd, plus accept an explicit `--manifest <path>` override for tests and unusual setups.

Documented as the single 🟡 polish finding in `docs/validation/2026-05-25_real_plc_two-controller_cross-binding_full-coverage.md`. Not blocking; user-facing ergonomic.

### Context to read first

- `docs/validation/2026-05-25_real_plc_two-controller_cross-binding_full-coverage.md` — the validation note that documented the finding, including the actual error message and workaround
- `examples/test_plc_full_coverage.rs` — Rust runner's current manifest-path resolution (look for the `PathBuf` construction in `parse_args`)
- `examples/CSharpFullCoverage/Program.cs` — C# runner's resolution in `Main` → `BuildTags(manifestPath)` (`Program.cs:251` is where it crashes)
- `python/examples/test_plc_full_coverage.py` — Python runner's resolution
- `examples/full_coverage_tags.json` — the manifest file itself; always lives at this path relative to the repo root
- `CODEX-AE` brief (already merged at `59a2176`) — established the manifest contract; this brief is pure ergonomics on top

### Files to create or modify

- `examples/test_plc_full_coverage.rs` — change the default manifest path computation to use `env!("CARGO_MANIFEST_DIR")` joined with `"examples/full_coverage_tags.json"`. `CARGO_MANIFEST_DIR` is set at compile time to the workspace member's directory; for the main crate that's the repo root. Keep `--manifest <path>` as an explicit override.
- `examples/CSharpFullCoverage/CSharpFullCoverage.csproj` — add a `<None Include="..\..\examples\full_coverage_tags.json">` with `<CopyToOutputDirectory>Always</CopyToOutputDirectory>` and `<Link>full_coverage_tags.json</Link>`. At build time MSBuild copies the manifest to `bin/Release/net10.0/`.
- `examples/CSharpFullCoverage/Program.cs` — change the default manifest path to `Path.Combine(AppContext.BaseDirectory, "full_coverage_tags.json")`. Keep `--manifest <path>` override.
- `python/examples/test_plc_full_coverage.py` — change the default manifest path to `Path(__file__).resolve().parents[2] / "examples" / "full_coverage_tags.json"`. `parents[2]` walks up from `python/examples/test_plc_full_coverage.py` → `python/examples` → `python` → repo root. Keep `--manifest <path>` override.
- `tests/full_coverage_manifest_tests.sh` — extend to invoke each runner with `--dry-run` from `/tmp` (a non-repo cwd), asserting each prints the expected `would-test` line. CI catches future regressions in the cwd-resolution behavior.
- `docs/validation/2026-05-25_real_plc_two-controller_cross-binding_full-coverage.md` — once this brief merges, update the "usability finding" paragraph to reflect that the gap is closed.

### Behavior

After this brief lands, each of these invocations succeeds without needing to be in the repo root:

```bash
# from /tmp
cd /tmp

cargo run --release --manifest-path /Users/sergiogallegos/projects/rust-ethernet-ip/Cargo.toml --example test_plc_full_coverage -- --dry-run
# → resolves manifest from CARGO_MANIFEST_DIR (the workspace member's dir)

dotnet run --project /Users/sergiogallegos/projects/rust-ethernet-ip/examples/CSharpFullCoverage -c Release -- --dry-run
# → resolves from AppContext.BaseDirectory (where the .dll lives, manifest copied next to it at build)

python3 /Users/sergiogallegos/projects/rust-ethernet-ip/python/examples/test_plc_full_coverage.py --dry-run
# → resolves from __file__'s repo root via parents[2]
```

`--manifest <path>` flag stays available on all three runners as an explicit override; tests use it for the fixture cases.

`--dry-run` output stays the same shape: `would-test binding={rust|csharp|python} tags=2299 writeable=2206 blocked=74 read_only=19`.

### Test requirements

- `tests/full_coverage_manifest_tests.sh` extended with three new test cases:
  - From `/tmp`, run each binding's `--dry-run` and assert it prints `would-test binding=X tags=2299`
  - From `/tmp`, run each binding's `--manifest /custom/path/to/full_coverage_tags.json --dry-run` and assert override works
  - Negative case: from `/tmp`, run with `--manifest /nonexistent/path` and assert clean error message (not a stack trace)
- Existing dry-run CI gate (`full-coverage-manifest` job) continues to pass.
- Rust + C# + Python full-coverage smoke tests still pass.
- `scripts/validate-agent-files` passes.

### Acceptance criteria

- All three runners successfully run `--dry-run` from any cwd (including `/tmp`, `~`, and subdirectories of the repo).
- Default manifest resolution is script-location-relative, not cwd-relative.
- `--manifest <path>` flag works on all three runners as explicit override.
- Bad manifest paths produce clear error messages, not stack traces.
- `tests/full_coverage_manifest_tests.sh` extended with the cwd-independence tests; CI passes.
- Validation evidence file updated to reflect the gap is closed.

### Out of scope

- Library code changes. This is `examples/` ergonomics only.
- Changing the manifest schema or output schema (CODEX-AE established those).
- Embedding the manifest as a compiled-in resource in any binding. Build-time copy (C#) and runtime path resolution (Rust, Python) are the right shape — the manifest stays a reviewable file that ships with the repo, not an opaque blob inside the binary.
- New runner flags beyond `--manifest`. Existing flags (`--dry-run`, `--skip-preflight`, etc.) stay as-is.
- Refactoring how the runners parse args. Just the manifest-resolution path.

### Risks and gotchas

- **C# `<Link>` element matters.** Without `<Link>full_coverage_tags.json</Link>` on the `<None Include>`, MSBuild copies the file to `bin/Release/net10.0/examples/full_coverage_tags.json` (preserving the original relative path). The `<Link>` flattens it to just `full_coverage_tags.json` next to the .dll, which is what `AppContext.BaseDirectory` resolves to.
- **Rust `env!("CARGO_MANIFEST_DIR")` is workspace-member-aware.** For an example in the main crate, it's the repo root. For an example in a sibling crate, it would be that sibling's directory. The full-coverage example is in the main crate so this is correct; don't accidentally move the example into a sibling crate without revisiting this.
- **Python `parents[2]` is path-length-sensitive.** If the script is ever moved (e.g. into a `python/examples/hardware/` subdirectory), the `parents[2]` would need to become `parents[3]`. Worth a one-line comment near the resolution: `# script lives at python/examples/<name>.py; walk up 2 levels to get repo root`.
- **Don't `git rev-parse --show-toplevel` as a fallback.** It requires git installed in the runtime environment (fine for dev, awkward for distributed scenarios). Stick with the language-native script-relative resolution.
- **Patch-release policy still holds.** This is `examples/` tooling, not a library change. Lands on `main` without triggering a 1.0.1 release per the 2026-05-25 maintainer direction.

## Codex log

### 2026-05-25  codex [gpt-5]

Implemented cwd-independent manifest resolution across all three full-coverage runners. Rust now resolves the default manifest from `CARGO_MANIFEST_DIR`; C# copies `full_coverage_tags.json` beside the built runner and resolves from `AppContext.BaseDirectory`; Python resolves from the script path back to the repo root. The existing `--manifest <path>` override remains available in all three runners.

Added clear `manifest-error:` handling for missing or malformed manifests so bad paths fail cleanly instead of producing C#/Python stack traces. Extended `tests/full_coverage_manifest_tests.sh` to run default, explicit-override, and negative bad-manifest dry-runs from `/tmp` for Rust, C#, and Python. Updated the 2026-05-25 two-controller validation note to mark the usability finding closed.

Verification:

- `tests/full_coverage_manifest_tests.sh` passed, including all `/tmp` cwd-independence cases.
- `cargo fmt -- --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `dotnet build examples/CSharpFullCoverage/CSharpFullCoverage.csproj -c Release` passed.
- `python3 -m py_compile python/examples/test_plc_full_coverage.py` passed.

## Claude review

_(append review entries here)_

## Verdict

_(final disposition)_
