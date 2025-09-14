# Mangle Wasm Wrapper

This crate provides a WebAssembly (Wasm) wrapper for the Mangle reasoning engine. It exposes a single function, `run_mangle_query`, which allows executing Mangle logic from JavaScript in a web browser.

## Prerequisites

Before building, you need to have the Rust toolchain and `wasm-pack` installed.

1.  **Install Rust:** If you don't have it, install Rust via [rustup](https://rustup.rs/).
2.  **Install wasm-pack:**
    ```sh
    cargo install wasm-pack
    ```

## Building the Wasm Module

To compile the crate into a Wasm module, navigate to this directory (`rust/mangle-wasm-wrapper`) and run the following command:

```sh
wasm-pack build --target web
```

This will create a `pkg` directory containing the compiled Wasm module, a JavaScript wrapper, and a TypeScript definition file. These files can be imported into any web application.

## Running the Native Tests

This wrapper includes a suite of native Rust tests to validate the core logic before compiling to WebAssembly. To run these tests, use the standard Cargo test command from within this directory:

```sh
cargo test
```

Since this project is part of a Cargo workspace, you can also run the tests from the root of the repository:

```sh
cargo test -p mangle-wasm-wrapper
```
