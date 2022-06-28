- [Rhai LSP](#rhai-lsp)
  - [Requirements](#requirements)
  - [Project Structure](#project-structure)
    - [`crates/rowan`](#cratesrowan)
    - [`crates/lsp`](#crateslsp)
    - [`crates/sourcegen`](#cratessourcegen)
    - [`editors/vscode`](#editorsvscode)
  - [Tests](#tests)
  - [Benchmarks](#benchmarks)
  - [Profiling](#profiling)
  - [Contributing](#contributing)
    - [Development Process](#development-process)
      - [Build and Install VSCode Extension](#build-and-install-vscode-extension)
      - [Build the Language Server](#build-the-language-server)
      - [Debugging the Language Server](#debugging-the-language-server)

# Rhai LSP

Experimental Rhai LSP Server and IDE support.

It's incomplete and not recommended for general use yet, everything can be subject to changes.

## Requirements

- Stable Rust toolchain (e.g. via [rustup](https://rustup.rs/))
- yarn (for VS Code)
- [vsce](https://www.npmjs.com/package/vsce) for VS Code extensions

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

### [`editors/vscode`](ide/vscode)

VS Code extension that uses the LSP.

If all the tools are available from the [Requirements](#requirements), it can be built and installed with `task ide:vscode:dev`.

## Tests

Run all tests with `cargo test`.

[Parser tests](crates/rowan/tests) are based on scripts found in [`testdata`](testdata), and also in the upstream [rhai submodule](rhai/scripts).

## Benchmarks

Run benchmarks with `cargo bench`.

Current parser results:

![bench](images/bench.png)

We can only go up from here. (although it is 3 times faster than a similar LALR generated parser)

## Profiling

To profile the parser, run `cargo bench --bench parse -- --profile-time 5`.

The flame graph outputs can be found in `target/criterion/profile` afterwards.

## Contributing

The documentation is still pretty much WIP (as everything else). All contributions are welcome!

### Development Process

Currently the following steps are used to develop the project via vscode:

#### Build and Install VSCode Extension

Install the extension with the following:
```sh
(cd editors/vscode && yarn && vsce package --no-yarn && code --install-extension *.vsix --force)
```

You only have to do this at the beginning or whenever you update the extension.

#### Build the Language Server

```sh
cargo install --path crates/lsp --debug
```

This will build and install the `rhai` executable globally that the vscode extension looks for.

After this step right now you have to manually kill the old running `rhai` executable or restart VSCode (`Developer: Reload Window`) in order for vscode to use the newly built language server.

#### Debugging the Language Server

The debugging process can consist of either strategically placed `tracing::info` statements that are visible in the VSCode debug console under `Rhai LSP`, or attaching a debugger to the running `rhai` process via [LLDB VSCode](https://marketplace.visualstudio.com/items?itemName=lanza.lldb-vscode). Both approaches deemed sufficient so far.
