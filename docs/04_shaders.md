# WGSL シェーダー解説

## WGSL とは

WGSL (WebGPU Shading Language) は WebGPU 専用のシェーダー言語です。
Rust に似た構文で、型安全性を重視しています。

## シェーダーの種類

### Vertex Shader（頂点シェーダー）

頂点ごとに実行され、3D 座標をクリップ座標に変換します。

```wgsl
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    // 頂点インデックスから座標を計算
    // 出力: クリップ座標 (-1 to 1)
}
```

### Fragment Shader（フラグメントシェーダー）

ピクセルごとに実行され、色を決定します。

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 出力: RGBA カラー (0.0 to 1.0)
    return vec4<f32>(1.0, 0.0, 0.0, 1.0); // 赤
}
```

## WGSL の基本構文

### データ型

| 型 | 説明 |
|----|------|
| `f32` | 32-bit 浮動小数点 |
| `i32` | 32-bit 符号付き整数 |
| `u32` | 32-bit 符号なし整数 |
| `bool` | 真偽値 |
| `vec2<f32>` | 2D ベクトル |
| `vec3<f32>` | 3D ベクトル |
| `vec4<f32>` | 4D ベクトル |
| `mat4x4<f32>` | 4x4 行列 |

### 構造体

```wgsl
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};
```

### 配列

```wgsl
var positions = array<vec2<f32>, 6>(
    vec2<f32>(-0.5, 0.5),
    vec2<f32>(0.5, 0.5),
    // ...
);
```

## Attributes（属性）

### @builtin

シェーダーに自動で渡される値:

| Builtin | 説明 |
|---------|------|
| `vertex_index` | 頂点インデックス (0, 1, 2, ...) |
| `instance_index` | インスタンスインデックス |
| `position` | 出力クリップ座標 |

### @location

頂点属性やフラグメント出力の場所を指定:

```wgsl
@location(0) color: vec3<f32>,  // Rust 側の vertex buffer と対応
```

## Phase 1: インスタンシングへの道

現在のシェーダーは頂点バッファなしで 1 つの四角形を描画しています。
10,000 個の四角形を描くには:

### 1. Instance 構造体を定義

```wgsl
struct Instance {
    @location(1) offset: vec2<f32>,
    @location(2) scale: vec2<f32>,
    @location(3) color: vec4<f32>,
};
```

### 2. Vertex Shader を修正

```wgsl
@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: Instance,
) -> VertexOutput {
    let base_pos = positions[vertex_index];
    let world_pos = base_pos * instance.scale + instance.offset;
    // ...
}
```

### 3. Rust 側で Instance Buffer を作成

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    offset: [f32; 2],
    scale: [f32; 2],
    color: [f32; 4],
}
```

## 学習課題

1. シェーダーの色を変えてみよう
2. 頂点座標を変えて形を変えてみよう
3. アニメーションを追加してみよう（Uniform Buffer が必要）

## デバッグ Tips

- `cargo run` でシェーダーエラーがコンソールに出力される
- 座標系: クリップ座標 (-1, -1) が左下、(1, 1) が右上
- 色: 0.0-1.0 の範囲（超えると clamp される）
