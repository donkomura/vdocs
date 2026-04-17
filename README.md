# vdocs

Google Docs で Vim 風の編集操作を提供する Chrome 拡張機能。

## Architecture

- **Rust/Wasm**: Vim のコアロジック (モード管理、キーシーケンス解析)
- **Chrome Extension (MV3)**: DOM インタラクション、キーイベント処理

## Prerequisites

```bash
# Install wasm-pack
cargo install wasm-pack
```

## Build

```bash
./scripts/build.sh
```

## Install

1. `chrome://extensions` を開く
2. 「デベロッパー モード」を有効化
3. 「パッケージ化されていない拡張機能を読み込む」をクリック
4. `extension/` ディレクトリを選択

## Development Status

現在 Phase 0 完了: ビルド基盤と空拡張の読み込み確認可能。
