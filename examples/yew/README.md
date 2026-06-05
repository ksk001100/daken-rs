# daken-rs Yew + Trunk example

This example uses Yew, `web-sys`, and Trunk.

## Run

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve examples/yew/index.html --open
```

Type romaji keys in the browser. Rejected keys increase the miss count without
changing the current matcher progress.
