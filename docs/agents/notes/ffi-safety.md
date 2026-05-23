# FFI Surface Maintainer Decisions

Use this page when reviewing or modifying `src/ffi.rs` or anything called across the C boundary by the C# wrapper. The FFI surface is what production .NET consumers run against. An unsound block here crashes the host process — there is no Rust panic boundary to catch it.

Verified against the current `src/ffi.rs` layout and the C# `RustEtherNetIp` P/Invoke surface.

## Every `unsafe` block carries a `// SAFETY:` comment

- Non-negotiable in this file. The comment names the invariant being upheld (e.g. "caller guarantees `ptr` is a valid `*mut c_char` returned from `eip_alloc_string` and not yet freed").
- If you can't write the SAFETY comment, the block is wrong. Find the invariant or restructure so the unsafe block isn't needed.
- Reviewers should reject any `unsafe` without a SAFETY comment, even one-liners.

## No `panic!`, `unwrap()`, `unreachable!` across the FFI boundary

- Panics unwinding into C are undefined behavior. Encode every failure as a return code (`EIP_ERROR_*`) instead.
- Use `Result<T, EtherNetIpError>` inside the crate and translate at the FFI boundary. The translation site is the only place that knows the C contract.
- Don't `.expect()` in a desperate "this can't fail" path — the FFI is exactly where unexpected failure happens (cold paths, init order, OS limits).

## Global runtime and client table are the contract

- The Tokio runtime lives at `crate::RUNTIME`, initialized via `std::sync::LazyLock`. All FFI calls `block_on` against this single runtime. Do not construct a per-call runtime — the C# wrapper opens many short-lived calls and per-call construction will exhaust threads.
- `FFI_CLIENTS: LazyLock<Mutex<HashMap<i32, EipClient>>>` is the only authoritative store of live `EipClient` handles. Integers (`i32`) cross the C boundary; `EipClient` instances never do.
- `FFI_NEXT_ID: LazyLock<Mutex<i32>>` is the single id allocator. Don't introduce a parallel allocator or reuse freed ids without coordination — the C# side caches handles and a reused id silently aliases two clients.

## Memory ownership across the boundary

- Strings returned to C are heap-allocated by Rust and freed by Rust (`eip_free_string`). Do not let the C# side `free()` them — allocator mismatch on Windows.
- Buffers passed in from C are borrowed for the duration of the call only. Don't store the raw pointer; copy into Rust-owned storage if it needs to outlive the call.
- Every "allocate" function in `ffi.rs` has a matching "free" function. When you add one, add the other in the same change. The C# wrapper relies on this pairing.

## When clippy flags `ffi.rs`

- `#[allow(clippy::missing_safety_doc)]` is not appropriate here. If clippy wants a doc on an `unsafe fn`, write one. The C# integrators read these docs through the generated bindings.
- Prefer `#[expect(<lint>, reason = "…")]` over `#[allow(…)]` so the suppression fails the build when the lint stops triggering. See the "Error handling and unsafe" section in `CLAUDE.md`.
