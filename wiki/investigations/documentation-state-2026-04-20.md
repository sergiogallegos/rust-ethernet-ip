# Documentation State 2026-04-20

## Summary

The repository documentation is in good shape for the current active surfaces, but it is not uniformly up to date across the full `docs/` tree.

- `confirmed`: The main active docs are reasonably aligned with current behavior.
- `needs-review`: Older secondary docs still contain pre-release `0.7.0` wording, references to removed wrapper trees, or historical implementation plans presented without enough framing.
- `confirmed`: The wiki is functioning well as a maintainer synthesis layer, but it should now be used to drive a targeted historical-doc cleanup pass.

## Current Understanding

### Active Docs That Look Healthy

- `README.md`
- `src/lib.rs` crate docs
- `docs/programmer_manual.md`
- `csharp/RustEtherNetIp/README.md`
- current validation records in `docs/validation/`
- current release draft / synthesis docs for `0.8.0`
- core wiki pages for limitations, route-path behavior, wrapper parity, and release validation

These appear consistent enough for active engineering and user guidance.

### Main Documentation Risks

#### Historical Docs Without Clear Framing

- `docs/ALL_WRAPPERS_UPDATE_COMPLETE.md`
- `docs/WRAPPER_UPDATE_SUMMARY.md`
- `docs/WRAPPER_LIMITATIONS_UPDATE_SUMMARY.md`
- `docs/DLL_DEPLOYMENT.md`
- `docs/LIBRARY_COMPARISON_AND_IMPROVEMENTS.md`
- `docs/UDT_DISCOVERY_v0.5.4.md`
- `docs/VERSION_0.6.0_CHANGELOG.md`

These are likely to confuse readers because they still contain removed-path references such as `pywrapper/` / `gowrapper/`, older roadmap states, or “production ready” claims from earlier implementation phases.

#### Pre-Release `0.7.0` Wording That Is Now Historical

- `docs/validation/REAL_PLC_TESTING.md`
- `docs/compat/0.7.0_plc_simulator_compatibility_matrix.md`
- `docs/0.7.0_HARDENING_GATE.md`
- `docs/audit/0.7.0_docs_api_audit.md`

These docs are still useful, but they describe the earlier hardening/release-prep state and should be marked or treated as historical reference material.

#### Validation Docs With Time-Bound “Current” Language

- `docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md`
- `docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`

These are evidence records, not user-facing overview docs. They remain valuable, but language like “current hardening status” is now historical and can mislead if read in isolation.

## Evidence

- `README.md`
- `docs/programmer_manual.md`
- `csharp/RustEtherNetIp/README.md`
- `wiki/index.md`
- `docs/validation/REAL_PLC_TESTING.md`
- `docs/compat/0.7.0_plc_simulator_compatibility_matrix.md`
- `docs/0.7.0_HARDENING_GATE.md`
- `docs/audit/0.7.0_docs_api_audit.md`
- `docs/ALL_WRAPPERS_UPDATE_COMPLETE.md`
- `docs/WRAPPER_UPDATE_SUMMARY.md`
- `docs/WRAPPER_LIMITATIONS_UPDATE_SUMMARY.md`
- `docs/DLL_DEPLOYMENT.md`
- `docs/LIBRARY_COMPARISON_AND_IMPROVEMENTS.md`
- `docs/UDT_DISCOVERY_v0.5.4.md`
- `docs/VERSION_0.6.0_CHANGELOG.md`

## Open Questions

- Whether to add a standard “historical reference” banner format and apply it repo-wide.
- Whether some old implementation-summary docs should be reframed instead of merely bannered.
- Whether `docs/README.md` should add a short section that distinguishes active docs from historical reference material.

## Related Pages

- [software-architecture-map.md](software-architecture-map.md)
- [python-wrapper-strategy-2026-04-19.md](python-wrapper-strategy-2026-04-19.md)
- [../wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
- [../releases/0.8.0-validation-synthesis.md](../releases/0.8.0-validation-synthesis.md)
