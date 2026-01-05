# Adamant - GPU-Accelerated Terminal Emulator

高性能な GPU 加速ターミナルエミュレーター。ゲームエンジンアーキテクチャを採用し、wgpu による Vulkan/Metal/DX12 レンダリングを実装。

**設計哲学**: "don't configure it. compile it." - 設定ファイルではなくコンパイル時に構成を決定

## クイックスタート

```bash
# 実行
cargo run

# リリースビルド（推奨）
cargo run --release

# テスト
cargo test

# ログ付き実行
RUST_LOG=debug cargo run
```

## ディレクトリ構造

```
src/
├── main.rs           # エントリポイント
├── lib.rs            # ライブラリ公開 API
├── app.rs            # アプリケーション状態 & イベントループ
└── renderer/
    ├── mod.rs        # GPU レンダラー（wgpu 初期化、描画）
    └── pipeline.rs   # レンダーパイプライン（シェーダー管理）

shaders/
└── quad.wgsl         # WGSL シェーダー

docs/                 # 設計ドキュメント
```

## 主要コンポーネント

- **App** (`src/app.rs`): winit の `ApplicationHandler` 実装。ウィンドウ/レンダラーのライフサイクル管理
- **Renderer** (`src/renderer/mod.rs`): wgpu サーフェス/デバイス/キュー管理、フレーム描画
- **Pipeline** (`src/renderer/pipeline.rs`): シェーダーパイプライン、描画コマンド発行

## 依存クレート

| クレート | 用途 |
|---------|------|
| wgpu | WebGPU 実装、GPU 操作 |
| winit | クロスプラットフォーム ウィンドウ管理 |
| pollster | 非同期ブロッキング実行 |
| bytemuck | GPU バッファ用バイト変換 |

## コーディング規約

- **アーキテクチャ**: ゲームループスタイル（MVC ではない）
- **非同期**: 初期化は `async fn`、`pollster::block_on()` で同期化
- **リソース管理**: `Arc<Window>` で共有所有権、`Option<T>` で初期化前状態
- **エラー型**: `Result<(), Box<dyn std::error::Error>>`
- **コメント**: `//!` でモジュール概要、`///` で公開 API、`// TODO:` でロードマップ

## 開発フェーズ

```
Phase 1: Window & Quad     ✅ 完了
  - ウィンドウ表示、wgpu パイプライン、インスタンシング
  - 詳細: docs/05_instancing.md

Phase 2: Text Engine       ← 現在
  - フォント読み込み、グリフラスタライズ、テクスチャアトラス

Phase 3: Terminal Logic
  - PTY 起動、キー入力処理、VT100/ANSI パーサー

Phase 4: Polish
  - IME 対応、ポストエフェクト、最適化
```

## ブランチ運用

- `main`: 安定版
- `work/phase-*`: フェーズごとの作業ブランチ
