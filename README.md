# daken-rs

[日本語 README](README.ja.md)

Typing-game friendly romaji input matcher for Japanese kana text.

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
- Common alternatives are accepted, including `shi`/`si`, `chi`/`ti`, `tsu`/`tu`.
- Yoon, small-kana spellings, doubled consonants for `っ`, and context-aware `ん`.
- `next_keys()` exposes currently valid next keys for UI hints.

## Quick API

- `RomajiInput::new(target)` creates a matcher from kana text.
- `input(char)` consumes one key and returns `Accepted`, `Completed`, or `Rejected`.
- `input_str(&str)` consumes a whole string until it is completed or rejected.
- `matches_romaji(target, input)` checks whether an input is one complete romanization.
- `confirmed_target_chars()` returns the confirmed target character count for UI highlighting.
- `confirmed_target_byte_index()` returns a byte index that can safely split `target()`.
- `target_parts()` returns the confirmed and unconfirmed target slices.
- `candidate_target_positions()` returns all current candidate target character positions.

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
