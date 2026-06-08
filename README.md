# daken-rs

[![Crates.io](https://img.shields.io/crates/v/daken-rs.svg)](https://crates.io/crates/daken-rs)
[![Docs.rs](https://docs.rs/daken-rs/badge.svg)](https://docs.rs/daken-rs)

[日本語 README](README.ja.md)

Typing-game friendly romaji input matcher for Japanese kana text.

## Installation

Add the crate with Cargo:

```bash
cargo add daken-rs
```

Or add it to `Cargo.toml` manually:

```toml
[dependencies]
daken-rs = "0.2"
```

Enable optional features when needed:

```toml
[dependencies]
daken-rs = { version = "0.2", features = ["serde", "wasm-bindgen"] }
```

The package name is `daken-rs`, and the Rust import path is `daken_rs`.

## Quick Start

```rust
use daken_rs::{KeyResult, RomajiInput};

let mut input = RomajiInput::new("かった");

assert_eq!(input.input('k'), KeyResult::Accepted);
assert_eq!(input.input('a'), KeyResult::Accepted);
assert_eq!(input.input('t'), KeyResult::Accepted);
assert_eq!(input.input('t'), KeyResult::Accepted);
assert_eq!(input.input('a'), KeyResult::Completed);
```

## Features

- Incremental key-by-key input judgement.
- Rejected keys do not mutate current progress.
- Hiragana and katakana targets are accepted.
- Full-width ASCII letters, numbers, spaces, and common symbols are accepted as normal keyboard input.
- Half-width kana, `ゐ` / `ゑ`, and additional common punctuation are normalized.
- Common alternatives are accepted, including `shi`/`si`, `chi`/`ti`, `tsu`/`tu`.
- Yoon, small-kana spellings, doubled consonants for `っ`, and context-aware `ん`.
- `next_keys()` exposes currently valid next keys for UI hints.
- `remaining_romaji_candidates()` exposes completion candidates from the current state.
- `TypingSession` can track misses and key history for game loops.

## Quick API

- `RomajiInput::new(target)` creates a matcher from kana text.
- `input(char)` consumes one key and returns `Accepted`, `Completed`, or `Rejected`.
- `input_str(&str)` consumes a whole string until it is completed or rejected.
- `matches_romaji(target, input)` checks whether an input is one complete romanization.
- `confirmed_target_chars()` returns the confirmed target character count for UI highlighting.
- `confirmed_target_byte_index()` returns a byte index that can safely split `target()`.
- `target_parts()` returns the confirmed and unconfirmed target slices.
- `candidate_target_positions()` returns all current candidate target character positions.
- `progress()` returns confirmed target characters, total target characters, typed key count, and completion state.
- `remaining_romaji_candidates()` returns romaji suffixes that can complete the target.
- `TypingSession::new(target)` creates a high-level session with miss count and input history.

## Feature flags

- `serde`: enables `Serialize` / `Deserialize` for `KeyResult`, `Progress`, `KeyStroke`, `TypingSession`, `RomajiInput`, and internal matcher state.
- `wasm-bindgen`: enables the JavaScript-friendly `WasmRomajiInput` wrapper.

## Miss Handling

Rejected keys do not mutate the internal matcher state.

```rust
use daken_rs::{KeyResult, RomajiInput};

let mut input = RomajiInput::new("かき");

assert_eq!(input.input('k'), KeyResult::Accepted);
assert_eq!(input.input('x'), KeyResult::Rejected);
assert_eq!(input.typed(), "k");

assert_eq!(input.input('a'), KeyResult::Accepted);
assert_eq!(input.input('k'), KeyResult::Accepted);
assert_eq!(input.input('i'), KeyResult::Completed);
```

## Examples

Console:

```bash
cargo run --manifest-path examples/console/Cargo.toml
```

WASM browser example:

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --manifest-path examples/wasm/Cargo.toml
mkdir -p examples/wasm/pkg
cp examples/wasm/target/wasm32-unknown-unknown/release/daken_wasm_example.wasm examples/wasm/pkg/
python -m http.server 8080 --directory examples/wasm
```

Then open http://localhost:8080.

Yew + Trunk WASM example:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve examples/yew/index.html --open
```
