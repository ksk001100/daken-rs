# daken-rs

[English README](README.md)

日本語タイピングゲーム向けの、ローマ字入力判定エンジンです。

```rust
use daken_rs::{KeyResult, RomajiInput};

let mut input = RomajiInput::new("かった");

assert_eq!(input.input('k'), KeyResult::Accepted);
assert_eq!(input.input('a'), KeyResult::Accepted);
assert_eq!(input.input('t'), KeyResult::Accepted);
assert_eq!(input.input('t'), KeyResult::Accepted);
assert_eq!(input.input('a'), KeyResult::Completed);
```

## 特徴

- 1キーずつ `Accepted` / `Completed` / `Rejected` を判定できます。
- ミスしたキーは進捗に反映されないため、ゲーム側でミス数だけ増やして続きから入力できます。
- ひらがな・カタカナのターゲットに対応しています。
- 全角英数字、全角スペース、よく使う全角記号を通常のキーボード入力として扱えます。
- 半角カナ、`ゐ` / `ゑ`、追加の句読点も正規化できます。
- `shi` / `si`、`chi` / `ti`、`tsu` / `tu` などの表記ゆれを受け付けます。
- 拗音、小書きかな、促音 `っ`、文脈依存の `ん` に対応しています。
- `next_keys()` で現在押せるキーを取得できます。
- `remaining_romaji_candidates()` で現在位置から完成までの候補を取得できます。
- `TypingSession` でミス数と入力履歴をまとめて管理できます。

## API

- `RomajiInput::new(target)` でターゲット文字列から判定器を作ります。
- `input(char)` で1キー入力し、`Accepted` / `Completed` / `Rejected` を返します。
- `input_str(&str)` で文字列をまとめて入力できます。
- `matches_romaji(target, input)` で入力文字列がターゲットの完全なローマ字入力か確認できます。
- `confirmed_target_chars()` で、かな target 側の確定済み文字数を取得できます。
- `confirmed_target_byte_index()` で、`target()` を安全に分割するための byte index を取得できます。
- `target_parts()` で、target の確定済み部分と未確定部分を取得できます。
- `candidate_target_positions()` で、現在の全候補状態が target のどの文字位置にいるかを取得できます。
- `progress()` で、確定済み文字数、全体文字数、入力済みキー数、完了状態をまとめて取得できます。
- `remaining_romaji_candidates()` で、現在の入力状態から完成までのローマ字候補を取得できます。
- `TypingSession::new(target)` で、ミス数と入力履歴つきの高レベルセッションを作れます。

## Feature flags

- `serde`: `KeyResult`、`Progress`、`KeyStroke`、`TypingSession`、`RomajiInput` などに `Serialize` / `Deserialize` を追加します。
- `wasm-bindgen`: JavaScript から直接使える `WasmRomajiInput` ラッパーを有効化します。

## ミス時の扱い

`Rejected` になったキーは内部状態に反映されません。

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

## サンプル

コンソール版:

```bash
cargo run --manifest-path examples/console/Cargo.toml
```

WASM 版:

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --manifest-path examples/wasm/Cargo.toml
mkdir -p examples/wasm/pkg
cp examples/wasm/target/wasm32-unknown-unknown/release/daken_wasm_example.wasm examples/wasm/pkg/
python -m http.server 8080 --directory examples/wasm
```

そのあと http://localhost:8080 を開いてください。

Yew + Trunk 版:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve examples/yew/index.html --open
```
