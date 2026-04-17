# vdocs

Google Docs で Vim 風の編集操作を提供する Chrome 拡張機能。

## Architecture

- **Rust/Wasm**: Vim のコアロジック (モード管理、キーシーケンス解析)
- **Chrome Extension (MV3)**: DOM インタラクション、キーイベント処理

## Build

```bash
make build
```

## Install

1. `chrome://extensions` を開く
2. 「デベロッパー モード」を有効化
3. 「パッケージ化されていない拡張機能を読み込む」をクリック
4. `extension/` ディレクトリを選択
