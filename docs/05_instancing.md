# GPU インスタンシング実装解説

## 概要

このドキュメントでは、Phase 1 で実装した GPU インスタンシングについて詳細に解説します。

## インスタンシングとは

### 問題: ドローコールのオーバーヘッド

ターミナルエミュレーターでは、画面に数千の文字セル（quad）を描画します。

```
80列 × 50行 = 4,000 個の quad
```

**素朴な方法**では、各 quad ごとに `draw()` を呼びます:

```rust
for quad in quads {
    render_pass.draw(quad);  // ← CPU→GPU 通信が毎回発生
}
```

これは **ドローコール**のオーバーヘッドで非常に遅くなります。GPU は高速ですが、CPU から GPU への命令送信にはコストがかかるためです。

### 解決策: インスタンシング

**同じジオメトリ**を**異なるパラメータ**で一度に描画する技術です。

```rust
// 1回の draw() で全 quad を描画
render_pass.draw(0..6, 0..4000);  // 6頂点 × 4000インスタンス
```

GPU は並列処理が得意なので、同じ形状を大量に描画する場合に威力を発揮します。

## アーキテクチャ

```
┌─────────────────────────────────────────────────────────────────┐
│                        GPU メモリ                               │
│                                                                 │
│  ┌─────────────────┐     ┌──────────────────────────────────┐  │
│  │  Vertex Buffer  │     │       Instance Buffer            │  │
│  │  (共有ジオメトリ) │     │       (インスタンスごとのデータ)    │  │
│  │                 │     │                                  │  │
│  │  v0: [0.0, 0.0] │     │  i0: pos[-0.9,-0.4] size[0.1]   │  │
│  │  v1: [1.0, 0.0] │     │  i1: pos[-0.78,-0.4] size[0.1]  │  │
│  │  v2: [0.0, 1.0] │     │  i2: pos[-0.66,-0.4] size[0.1]  │  │
│  │  v3: [1.0, 0.0] │  ×  │  ...                            │  │
│  │  v4: [1.0, 1.0] │     │  i127: pos[0.78,0.28] size[0.1] │  │
│  │  v5: [0.0, 1.0] │     │                                  │  │
│  └─────────────────┘     └──────────────────────────────────┘  │
│         │                           │                          │
│         └─────────┬─────────────────┘                          │
│                   ▼                                            │
│          ┌───────────────┐                                     │
│          │ Vertex Shader │ ← 6頂点 × 128インスタンス = 768回実行 │
│          └───────────────┘                                     │
│                   │                                            │
│                   ▼                                            │
│          ┌────────────────┐                                    │
│          │ Fragment Shader │ ← ピクセルごとに実行               │
│          └────────────────┘                                    │
└─────────────────────────────────────────────────────────────────┘
```

## 実装詳細

### 1. シェーダー (`shaders/quad.wgsl`)

#### データ構造

```wgsl
// 頂点バッファから読み取るデータ
struct VertexInput {
    @location(0) position: vec2<f32>,  // ローカル座標 [0,1]
};

// インスタンスバッファから読み取るデータ
struct InstanceInput {
    @location(1) pos: vec2<f32>,    // クリップ空間での位置
    @location(2) size: vec2<f32>,   // クリップ空間でのサイズ
    @location(3) color: vec4<f32>,  // RGBA カラー
};
```

#### 頂点シェーダー

```wgsl
@vertex
fn vs_main(
    vertex: VertexInput,      // 頂点バッファから (6頂点を循環)
    instance: InstanceInput,  // インスタンスバッファから (インスタンスごとに異なる)
) -> VertexOutput {
    var out: VertexOutput;

    // ローカル座標 [0,1] をインスタンスのサイズでスケール＆位置でオフセット
    let world_pos = vertex.position * instance.size + instance.pos;

    out.clip_position = vec4<f32>(world_pos, 0.0, 1.0);
    out.color = instance.color;

    return out;
}
```

**ポイント:**
- `vertex.position` は単位 quad の座標（[0,0] ~ [1,1]）
- `instance.size` でスケーリング
- `instance.pos` でオフセット（左下基準）

### 2. Rust 側のデータ構造 (`src/renderer/pipeline.rs`)

#### 頂点構造体

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}
```

#### インスタンス構造体

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub pos: [f32; 2],   // クリップ空間での位置 (-1.0 ~ 1.0)
    pub size: [f32; 2],  // クリップ空間でのサイズ
    pub color: [f32; 4], // RGBA (0.0 ~ 1.0)
}
```

**`#[repr(C)]`**: C 言語と同じメモリレイアウトを保証。GPU との互換性のため必須。

**`bytemuck::Pod`**: "Plain Old Data" - 安全にバイト列として扱える型であることを示す。

#### 頂点バッファレイアウト

```rust
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] =
        wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,  // ← 頂点ごとに進む
            attributes: &Self::ATTRIBS,
        }
    }
}
```

#### インスタンスバッファレイアウト

```rust
impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        1 => Float32x2,  // pos   → @location(1)
        2 => Float32x2,  // size  → @location(2)
        3 => Float32x4,  // color → @location(3)
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,  // ← インスタンスごとに進む
            attributes: &Self::ATTRIBS,
        }
    }
}
```

**`step_mode` の違い:**
- `Vertex`: 頂点ごとにバッファを進める（0→1→2→3→4→5→0→1→...）
- `Instance`: インスタンスが切り替わるときだけ進める

### 3. バッファ作成

#### 頂点バッファ（静的・共有）

```rust
const QUAD_VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.0] }, // bottom-left
    Vertex { position: [1.0, 0.0] }, // bottom-right
    Vertex { position: [0.0, 1.0] }, // top-left
    Vertex { position: [1.0, 0.0] }, // bottom-right
    Vertex { position: [1.0, 1.0] }, // top-right
    Vertex { position: [0.0, 1.0] }, // top-left
];

let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Vertex Buffer"),
    contents: bytemuck::cast_slice(QUAD_VERTICES),
    usage: wgpu::BufferUsages::VERTEX,
});
```

#### インスタンスバッファ（動的更新可能）

```rust
let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Instance Buffer"),
    contents: bytemuck::cast_slice(&instances),
    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
});
```

**`COPY_DST`**: `queue.write_buffer()` でデータを更新可能にするフラグ。
Phase 3 でターミナル出力に応じてセルの色を変える際に使用。

### 4. パイプライン設定

```rust
let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    // ...
    vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        buffers: &[Vertex::desc(), Instance::desc()],  // ← 2つのバッファレイアウト
        // ...
    },
    // ...
});
```

### 5. 描画

```rust
pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));   // slot 0
    render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..)); // slot 1

    // 6頂点 × instance_count インスタンス
    render_pass.draw(0..6, 0..self.instance_count);
}
```

## テストデータ生成

デバッグ用に 16×8 のカラフルなグリッドを生成:

```rust
fn create_test_instances() -> Vec<Instance> {
    let cols = 16;
    let rows = 8;
    let quad_size = 0.1;
    let spacing = 0.12;

    let mut instances = Vec::with_capacity(cols * rows);

    for row in 0..rows {
        for col in 0..cols {
            let x = (col as f32 - cols as f32 / 2.0) * spacing;
            let y = (row as f32 - rows as f32 / 2.0) * spacing;

            // 位置に応じた虹色
            let r = col as f32 / cols as f32;
            let g = row as f32 / rows as f32;
            let b = 1.0 - (r + g) / 2.0;

            instances.push(Instance {
                pos: [x, y],
                size: [quad_size, quad_size],
                color: [r, g, b, 1.0],
            });
        }
    }

    instances
}
```

## 座標系

```
        クリップ空間
    (-1, 1) ──────── (1, 1)
       │              │
       │   (0, 0)     │
       │      ●       │
       │              │
   (-1,-1) ──────── (1,-1)
```

- 原点 (0, 0) は画面中央
- X: 左 -1 → 右 +1
- Y: 下 -1 → 上 +1

## パフォーマンス

| 方式 | ドローコール | CPU→GPU 転送 | 相対速度 |
|------|-------------|-------------|---------|
| 個別描画 | N 回 | N 回 | 1x |
| インスタンシング | 1 回 | 1 回 | 50-100x |

4,000 quad の場合、インスタンシングにより約 50-100 倍の高速化が期待できます。

## 次のステップ (Phase 2)

テキストレンダリングのためにインスタンスデータを拡張:

```rust
pub struct Instance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub uv_offset: [f32; 2],  // グリフアトラス内の UV オフセット
    pub uv_size: [f32; 2],    // グリフの UV サイズ
}
```

シェーダーでテクスチャサンプリング:

```wgsl
@group(0) @binding(0) var glyph_sampler: sampler;
@group(0) @binding(1) var glyph_atlas: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(glyph_atlas, glyph_sampler, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
```
