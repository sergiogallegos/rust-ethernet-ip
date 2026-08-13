# Program-Scoped Tag Discovery

## Summary

`needs-review` as of 2026-08-13: PR #28 merged as `afac5ee`, fixing missing
Symbol Object pagination in `discover_program_tags`. PR #29 addresses the
separate incorrect-`TagScope` defect, but remains open and conflicting until it
is rebased and program-name normalization is added.

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
  scope. Current parsing hardcodes every result to `TagScope::Controller`; the
  request caller must supply the scope, as proposed in PR #29.
- `needs-review`: `discover_program_tags` accepts either `"Dashboard"` or
  `"Program:Dashboard"`, but PR #29 constructs `TagScope::Program` from the raw
  argument. The prefixed form would therefore produce
  `Program("Program:Dashboard")`, contrary to repository examples and schema
  tests that store only the program name. Normalize once and use the normalized
  name for both the request and returned scope.
- `confirmed`: PR #29 still targets the pre-#28 base and GitHub reports it as
  conflicting. Its rebase should retain the scope parameter on
  `parse_tag_list_response_page` and both scope-aware call sites; the private
  wrapper removed by #28 should stay removed.

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
- Local review on 2026-08-13: formatting, focused discovery tests, targeted
  program-tag tests, and all-feature Clippy passed independently for both PR
  heads. The all-workspace/all-target run exceeded the review timeout after
  many passing suites and produced no PR-specific failure before timeout.

## Open Questions

- `likely non-blocking`: PR #28 inherits the controller discovery helper's
  16-bit start-instance ceiling. A 32-bit logical instance segment should be
  considered for both discovery paths in one follow-up rather than changed in
  only the program path.
- `needs-review`: Neither PR has an automated end-to-end pagination test that
  feeds multiple Symbol Object replies through `discover_program_tags`; PR #28
  has request-builder coverage plus real-hardware evidence.
- `unconfirmed`: The contributor's CompactLogix firmware revision was not
  recorded, so firmware-specific comparison is not possible from the PR alone.

## Related Pages

- [firmware behavior](../controllers/firmware-behavior.md)
- [CIP path validation](cip-path-validation.md)
- [0.8.0 validation synthesis](../releases/0.8.0-validation-synthesis.md)
