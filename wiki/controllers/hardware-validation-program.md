# Hardware Validation Program

## Summary

The user-facing matrix and test protocol live in
[`docs/HARDWARE_COMPATIBILITY.md`](../../docs/HARDWARE_COMPATIBILITY.md). This
page records the maintainer interpretation behind it.

## Current Understanding

- `confirmed`: Five exact processor/firmware combinations have physical
  evidence, but only the 5069-L330ERM fw38 row is a `1.2.0` four-binding gate.
- A `Done` cell means an authoritative validation file exists for that exact
  binding and target; blank cells are invitations, not implied failures.
- Functional, endurance, and performance claims are separate. Passing one
  full-coverage run does not establish 24-hour stability or a universal
  throughput number.
- Performance records must include topology, sample count, latency
  distribution, errors, and resource/controller impact to support comparison.
- Any write-heavy contribution must identify a test-only program, starting
  values, and final restore/settle state.

## Evidence

- [Hardware compatibility and test program](../../docs/HARDWARE_COMPATIBILITY.md)
- [Result template](../../docs/validation/REAL_HARDWARE_RESULT_TEMPLATE.md)
- [1.2.0 validation synthesis](../releases/1.2.0-validation-synthesis.md)

## Open Questions

- Select a second routinely available release-gate processor.
- Add the first C/C++ results on a ControlLogix target.
- Add the first 24-hour result with latency percentiles, data-gap detection,
  RSS/CPU trend, and reconnect accounting.

## Related Pages

- [firmware-behavior.md](firmware-behavior.md)
- [../protocol/route-path-behavior.md](../protocol/route-path-behavior.md)
