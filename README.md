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

### リリース版からインストール

1. [Releases](https://github.com/donkomura/vdocs/releases) から最新の `vdocs-<tag>.zip` をダウンロード
2. 任意のディレクトリに展開する（中に `extension/` ディレクトリが含まれる）
3. `chrome://extensions` を開く
4. 「デベロッパー モード」を有効化
5. 「パッケージ化されていない拡張機能を読み込む」をクリック
6. 展開した `extension/` ディレクトリを選択

> Chrome ウェブストア未配布のため、現在は手動インストールのみ対応。拡張機能を更新する場合は、新しい zip を展開したうえで `chrome://extensions` の「更新」ボタンを押す。

### ローカルビルドからインストール

1. `make build` で `extension/pkg/` を生成
2. `chrome://extensions` を開く
3. 「デベロッパー モード」を有効化
4. 「パッケージ化されていない拡張機能を読み込む」をクリック
5. `extension/` ディレクトリを選択
