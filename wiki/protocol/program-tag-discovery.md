# Program-Scoped Tag Discovery

## Summary

`confirmed` as of 2026-08-14: PR #28 merged as `afac5ee`, fixing missing
Symbol Object pagination in `discover_program_tags`. PR #29 then merged as
`481f20d`, addressing the separate incorrect-`TagScope` defect with normalized
program names and regression coverage for both accepted input forms.

## Current Understanding

- `confirmed`: Program-scoped Symbol Object enumeration uses the same `0x55`
  Get Instance Attribute List paging contract as controller-scoped discovery.
  General status `0x06` carries a partial page; enumeration resumes from the
  last returned instance plus one.
- `confirmed`: Before PR #28, `discover_program_tags` made one request and
  checked `0x06` as a fatal CIP error. Squash commit `afac5ee` now mirrors the
  existing controller-scope loop, parameterizes the program request's start
  instance, and adds overflow and stalled-pagination guards.
- `confirmed`: Symbol Object replies do not encode controller-versus-program
  scope. Before PR #29, parsing hardcoded every result to
  `TagScope::Controller`; current code supplies the scope held by the request
  caller.
- `confirmed`: PR #29 centralizes the accepted-name normalization in
  `program_scope_name`. Both `"Dashboard"` and `"Program:Dashboard"` now build
  the same request path and produce `TagScope::Program("Dashboard")`.
- `confirmed`: PR #29 retains the scope parameter on
  `parse_tag_list_response_page`, passes controller scope from controller
  discovery, and hoists normalized program scope outside the program paging
  loop. The private wrapper removed by PR #28 remains removed.
- `confirmed`: PR #29 was squash-merged as `481f20d` on 2026-08-14. Its PR CI
  completed successfully across all 29 jobs, including stable and beta Rust
  jobs on Windows, Ubuntu, and macOS; the post-merge `main` workflow also passed.
  Stable jobs include the C# unit and native P/Invoke integration suites.

## Evidence

- [`src/client.rs`](../../src/client.rs) contains the paged controller-scope
  implementation, the single-page program implementation, accepted prefixed
  program-name form, and hardcoded controller scope.
- [`docs/agents/tasks/CODEX-AM-tag-addressing-correctness.md`](../../docs/agents/tasks/CODEX-AM-tag-addressing-correctness.md)
  records the request-path correction that made program-scoped Symbol Object
  enumeration reachable.
- [`docs/validation/2026-07-02_tag_addressing_smoke_5069-L330ERM_fw38.md`](../../docs/validation/2026-07-02_tag_addressing_smoke_5069-L330ERM_fw38.md)
  confirms that the public API is exercised with the accepted
  `Program:TestProgram` form.
- [PR #28](https://github.com/sergiogallegos/rust-ethernet-ip/pull/28), merged
  as `afac5ee`, reports CompactLogix 5380 hardware evidence: the published
  version failed with `0x06`, while the patched pair returned the expected tags.
- [PR #29](https://github.com/sergiogallegos/rust-ethernet-ip/pull/29)
  reports hardware evidence for program-scope labeling with both patches
  applied.
- Independent local review of PR #29 head `fcedeb4` on 2026-08-14: formatting,
  all-target Clippy with warnings denied, and all 16 focused discovery tests
  passed. A release FFI build and `RustEtherNetIp.Tests` also passed all 86 C#
  tests on .NET 10. Replacing `program_scope_name` with an identity function
  made exactly the two new accepted-name regression tests fail while the other
  14 discovery tests remained green.

## Open Questions

- `likely non-blocking`: PR #28 inherits the controller discovery helper's
  16-bit start-instance ceiling. A 32-bit logical instance segment should be
  considered for both discovery paths in one follow-up rather than changed in
  only the program path.
- `likely non-blocking`: Neither PR has an automated end-to-end pagination test that
  feeds multiple Symbol Object replies through `discover_program_tags`; PR #28
  has request-builder coverage plus real-hardware evidence.
- `unconfirmed`: The contributor's CompactLogix firmware revision was not
  recorded, so firmware-specific comparison is not possible from the PR alone.

## Related Pages

- [firmware behavior](../controllers/firmware-behavior.md)
- [CIP path validation](cip-path-validation.md)
- [0.8.0 validation synthesis](../releases/0.8.0-validation-synthesis.md)
