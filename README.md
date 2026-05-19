# Love

A small Rust/WebAssembly heart animation that opens as a static web page.

## Requirements

- Rust stable with the `wasm32-unknown-unknown` target
- `wasm-bindgen-cli`

## Setup

```sh
rustup update
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.121
```

## Build

```sh
./scripts/build.sh
```

Serve the static output locally:

```sh
python3 -m http.server 8000 --directory dist
```

Then open `http://localhost:8000`.
