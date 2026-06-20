---
id: CODEX-AI
title: Publish a manylinux Linux x86_64 Python wheel to PyPI
owner: codex
status: open
created: 2026-06-19
last-update: 2026-06-19 claude [Opus 4.8]
---

## Brief

### Goal

Make `pip install rust-ethernet-ip` work on Linux x86_64 by publishing a
PyPI-acceptable **manylinux** wheel with the native library bundled. Today the
1.1.0 release ships Windows (`win_amd64`) and macOS (`macosx_*`) wheels plus an
sdist, but **no Linux wheel** — so on Linux pip falls back to the sdist, which
contains no native library and fails at runtime with `NativeLibraryLoadError`.

### Context to read first

- `python/setup.py` — the shim that forces a platform-tagged wheel
  (`BinaryDistribution.has_ext_modules` + `bdist_wheel.root_is_pure = False` +
  `get_tag` → `py3-none-<plat>`).
- `python/pyproject.toml` — `[tool.setuptools.package-data]` bundles the staged
  `*.so`/`*.dll`/`*.dylib`; `python/scripts/stage_native_lib.py` copies the
  cargo-built cdylib into the package before `python -m build`.
- `python/rust_ethernet_ip/bindings.py` `_candidate_paths` — loads the native
  lib from inside the installed package first.
- `.github/workflows/release.yml` — the `build-native`, `pypi-wheels`, and
  `pypi-publish` jobs. `pypi-wheels` currently has `linux-x64` **omitted** from
  its matrix (with a NOTE explaining why). `build-native` already produces a
  `linux-x64` native artifact (`librust_ethernet_ip.so`).

### Why the previous attempt failed (root cause to fix)

The first release.yml attempt built the Linux wheel from the prebuilt cdylib and
ran `auditwheel repair`. `auditwheel` rejected it:

```
RuntimeError: Invalid binary wheel, found the following shared library/libraries
in purelib folder: librust_ethernet_ip.so
The wheel has to be platlib compliant in order to be repaired by auditwheel.
```

So the wheel marked the package as **purelib** (pure-Python) even though it
contains a `.so`. auditwheel only repairs **platlib** wheels. The
`has_ext_modules = True` / `root_is_pure = False` overrides in `setup.py`
produced a correct *platform tag* locally and on Windows/macOS, but the Linux
build still landed the `.so` in purelib. Determine why and make the wheel
genuinely platlib (`Root-Is-Purelib: false` in the `WHEEL` metadata, package
under platlib), so auditwheel can repair it to a `manylinux_*` tag.

### Behavior / approach

Re-enable a Linux wheel in `release.yml` that PyPI will accept. Either approach
is acceptable; pick the one that is most robust:

1. **Build + repair inside a manylinux container.** Run the Linux `pypi-wheels`
   matrix leg in `quay.io/pypa/manylinux_2_28_x86_64` (or `_2_34`). Build the
   cdylib with a Rust toolchain inside the container (so it links that
   container's glibc), stage it, build the wheel, then `auditwheel repair` to
   the manylinux tag. This guarantees a compliant glibc floor.
2. **Fix the platlib layout, then repair on ubuntu-latest.** If you can make the
   wheel platlib-compliant (so auditwheel accepts it), you can keep building on
   `ubuntu-latest` and repair there — but the resulting manylinux tag will track
   `ubuntu-latest`'s glibc (newer = narrower compatibility). Less portable;
   prefer (1) unless (2) is clearly sufficient.

Re-add `linux-x64` to the `pypi-wheels` matrix and restore the (corrected)
manylinux repair step. Keep Windows/macOS legs unchanged.

### Test requirements

- `auditwheel show <wheel>` reports a `manylinux_*_x86_64` platform tag (not
  `linux_x86_64`).
- `twine check dist/*` passes for the Linux wheel.
- In a clean Linux environment (e.g. a `python:3.12-slim` container, NOT the
  build container): `pip install <wheel>` then
  `python -c "import rust_ethernet_ip as r; c = r.Client('127.0.0.1:1'); print('loaded')"`
  must load the native library (a connection failure is fine; a
  `NativeLibraryLoadError` is not).
- A CI smoke step that installs the freshly built Linux wheel and imports it
  (mirror the existing `package` smoke in `ci.yml` that installs the wheel).

### Acceptance criteria

- [ ] A `v*` tag (or `workflow_dispatch`) publishes a `manylinux_*_x86_64` wheel
      for CPython-agnostic (`py3-none-manylinux...`) `rust-ethernet-ip` to PyPI.
- [ ] `pip install rust-ethernet-ip` on Linux x86_64 yields a working package
      (native library loads), verified by a CI install-and-import smoke.
- [ ] `release.yml` `pypi-wheels` matrix includes `linux-x64` again; the
      deferral NOTE is removed.
- [ ] Windows/macOS wheels and the sdist still publish unchanged.

### Out of scope

- Linux `aarch64` (arm64) wheels — separate follow-up if demand exists.
- musl/Alpine wheels.
- Windows and macOS wheels (already shipping and working).

### Risks / gotchas

- The manylinux tag is determined by the **glibc of the build environment**. Use
  a manylinux container to get a low, portable floor (`manylinux_2_28` covers
  most current distros).
- The Rust cdylib links `libc`, `libpthread`, `libgcc_s`, `libm`, `libdl` —
  all standard and bundled/whitelisted by auditwheel; there should be no
  external non-system deps to vendor.
- Do not regress the platform tag of the Windows/macOS wheels (their `get_tag`
  override in `setup.py` must stay `py3-none-<plat>`).
- 1.1.0 is already published to PyPI **without** a Linux wheel; this work lands
  the Linux wheel in the **next** release (e.g. 1.1.1 or 1.2.0). PyPI versions
  are immutable — do not attempt to re-upload 1.1.0.

## Codex log

## Claude review

## Verdict
