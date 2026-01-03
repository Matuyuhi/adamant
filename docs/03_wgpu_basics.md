# wgpu 基礎解説

## wgpu とは

wgpu は WebGPU 仕様の Rust 実装です。プラットフォームごとに以下のバックエンドを使用:

- **Windows**: DirectX 12 / Vulkan
- **macOS**: Metal
- **Linux**: Vulkan
- **Web**: WebGPU (ブラウザ)

## 主要な概念

### 1. Instance, Adapter, Device

```
┌──────────┐
│ Instance │  ← wgpu 全体のエントリーポイント
└────┬─────┘
     │ request_adapter()
     ▼
┌──────────┐
│ Adapter  │  ← 物理 GPU を表す
└────┬─────┘
     │ request_device()
     ▼
┌──────────┐
│  Device  │  ← 論理デバイス（実際に使う GPU 抽象）
│  Queue   │  ← コマンド送信キュー
└──────────┘
```

### 2. Surface と SwapChain

```rust
// Surface = ウィンドウに紐づいた描画ターゲット
let surface = instance.create_surface(window)?;

// 毎フレーム、次のテクスチャを取得して描画
let output = surface.get_current_texture()?;
```

### 3. Render Pipeline

シェーダーをどう実行するかを定義:

```rust
device.create_render_pipeline(&RenderPipelineDescriptor {
    vertex: VertexState { ... },     // 頂点シェーダー
    fragment: FragmentState { ... }, // フラグメントシェーダー
    primitive: PrimitiveState { ... }, // プリミティブの種類
    ...
})
```

### 4. Command Encoder

GPU へのコマンドをバッファリング:

```rust
let mut encoder = device.create_command_encoder(...);

{
    let mut render_pass = encoder.begin_render_pass(...);
    render_pass.set_pipeline(&pipeline);
    render_pass.draw(0..6, 0..1);
}

queue.submit(std::iter::once(encoder.finish()));
```

## レンダリングの流れ

```
1. get_current_texture()  ← Surface から描画先を取得
2. create_command_encoder() ← コマンドバッファ作成
3. begin_render_pass()    ← 描画パス開始
4. set_pipeline()         ← 使用するパイプライン設定
5. draw()                 ← 描画コマンド追加
6. end render_pass        ← 描画パス終了
7. queue.submit()         ← GPU に送信
8. present()              ← 画面に表示
```

## 学習課題

1. `adapter.get_info()` で自分の GPU 情報を確認しよう
2. クリアカラーを変えて背景色を変更しよう
3. `wgpu::Limits` で GPU の制限を確認しよう

## 参考リンク

- [wgpu 公式チュートリアル](https://sotrh.github.io/learn-wgpu/)
- [WebGPU 仕様](https://www.w3.org/TR/webgpu/)
