# イベントループ解説

## winit のイベントループ

winit 0.30 では `ApplicationHandler` トレイトを実装してイベントを処理します。

```rust
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // アプリ起動時・再開時に呼ばれる
        // ここでウィンドウと GPU リソースを初期化
    }

    fn window_event(&mut self, ..., event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => { /* 終了処理 */ }
            WindowEvent::Resized(size) => { /* リサイズ処理 */ }
            WindowEvent::RedrawRequested => { /* 描画処理 */ }
            WindowEvent::KeyboardInput { .. } => { /* 入力処理 */ }
            _ => {}
        }
    }
}
```

## イベントの流れ

1. **Resumed**: アプリ起動、ウィンドウ作成
2. **RedrawRequested**: 毎フレーム呼ばれる（`request_redraw()` でトリガー）
3. **KeyboardInput**: キー入力イベント
4. **CloseRequested**: 閉じるボタンが押された

## 重要なポイント

### VSync とフレームレート

```rust
present_mode: wgpu::PresentMode::Fifo,  // VSync ON (60fps に制限)
```

| PresentMode | 説明 |
|-------------|------|
| `Fifo` | VSync ON。画面のリフレッシュレートに同期 |
| `Immediate` | VSync OFF。可能な限り高速に描画（ティアリング発生可能）|
| `Mailbox` | VSync ON だがフレーム落ちを最小化 |

### 非同期 vs 同期

現在は `pollster::block_on()` で同期的に wgpu を初期化しています。
これは起動時のブロッキングを意味しますが、シンプルさを優先しています。

## 学習課題

1. `WindowEvent` の他のイベントを調べてみよう
2. `PresentMode::Immediate` に変えてフレームレートの変化を観察しよう
3. キー入力をログに出力してみよう（`RUST_LOG=trace cargo run`）
