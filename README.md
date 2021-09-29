# Rhai LSP

Rhai LSP Server and IDE support.

## Requirements

- Stable Rust toolchain (e.g. via [rustup](https://rustup.rs/))
- [vsce](https://www.npmjs.com/package/vsce) for VS Code extensions

**Optionally:**

- [Task](https://taskfile.dev) for running predefined tasks.

## Project Structure

### [`crates/rowan`](crates/rowan)

Rhai syntax and a recursive descent parser based on [Rowan](https://github.com/rust-analyzer/rowan).

The high-level syntax ([ungrammar](https://rust-analyzer.github.io/blog/2020/10/24/introducing-ungrammar.html)) definition is found in [crates/rowan/src/ast/rhai.ungram](crates/rowan/src/ast/rhai.ungram). The parser mimics the structure and produces a fitting CST.

### [`crates/lsp`](crates/lsp)

The LSP server implementation backed up by [lsp-async-stub](https://github.com/tamasfe/taplo/tree/master/lsp-async-stub).

It can be compiled to WASM only right now, but native binaries with stdio or TCP communication can be easily implemented.

### [`crates/sourcegen`](crates/sourcegen)

Crate for source generation.

Currently only some node types and helper macros are generated from the ungrammar definition. Later the AST will also be generated from it.

### [`js/lsp`](js/lsp)

A JavaScript wrapper over the LSP so that it can be used in NodeJS (and browser) environments, it greatly improves portability (e.g. the same JS *"binary"* can be used in a VS Code extension or with coc.nvim).

### [`ide/vscode`](ide/vscode)

VS Code extension that uses the LSP.

If all the tools are available from the [Requirements](#requirements), it can be built and installed with `task vscode:dev`.

### [`ide/web`](ide/web)

A web demo page, nice to have in the future, currently has an editor and does nothing.

## Tests

Run all tests with `cargo test`.

[Parser tests](crates/rowan/tests) are based on scripts found in [`testdata`](testdata), and also in the upstream [rhai submodule](rhai/scripts).

## Benchmarks

Run benchmarks with `cargo bench`.

Current parser results:

![bench](images/bench.png)

We can only go up from here.

## Profiling

To profile the parser, run `cargo bench --bench parse -- --profile-time 5`.

The flame graph outputs will can found in `target/criterion/profile` afterwards.
