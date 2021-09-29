`no-std` Test Sample
====================

This sample application is a bare-bones `no-std` build for testing.

[`wee_alloc`](https://crates.io/crates/wee_alloc) is used as the allocator.


To Build
--------

The nightly compiler is required:

```bash
cargo +nightly build --release
```

A specific profile can also be used:

```bash
cargo +nightly build --profile unix -Z unstable-options
```

Three profiles are defined: `unix`, `windows` and `macos`.

The release build is optimized for size.  It can be changed to optimize on speed instead.
