# daken-rs WASM example

This example uses plain `wasm32-unknown-unknown` exports, so it does not need
`wasm-bindgen` or npm dependencies.

## Build

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --manifest-path examples/wasm/Cargo.toml
```

Copy the generated `.wasm` into the browser app directory:

```bash
mkdir -p examples/wasm/pkg
cp examples/wasm/target/wasm32-unknown-unknown/release/daken_wasm_example.wasm examples/wasm/pkg/
```

Serve the directory with any static file server:

```bash
python -m http.server 8080 --directory examples/wasm
```

Then open http://localhost:8080.

## Yew + Trunk alternative

There is also a Yew + Trunk browser example in `examples/yew`:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve examples/yew/index.html --open
```
