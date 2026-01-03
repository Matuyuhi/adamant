// Adamant Quad Shader
//
// This shader renders a simple colored quad on screen.
// See docs/04_shaders.md for WGSL basics.
//
// WGSL (WebGPU Shading Language) is a new shading language designed for WebGPU.
// It's similar to GLSL/HLSL but with Rust-like syntax.

// Vertex shader output / Fragment shader input
struct VertexOutput {
    // @builtin(position) marks this as the clip-space position (required)
    @builtin(position) clip_position: vec4<f32>,
    // Custom varying - color to pass to fragment shader
    @location(0) color: vec3<f32>,
};

// Vertex shader
// Generates a quad using vertex indices (no vertex buffer needed for now)
@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    // Define quad vertices in clip space (-1 to 1)
    // Two triangles forming a rectangle
    //
    //  0 --- 1
    //  |   / |
    //  | /   |
    //  2 --- 3
    //
    // Triangle 1: 0, 1, 2
    // Triangle 2: 2, 1, 3

    var positions = array<vec2<f32>, 6>(
        // Triangle 1
        vec2<f32>(-0.5, 0.5),   // 0: top-left
        vec2<f32>(0.5, 0.5),    // 1: top-right
        vec2<f32>(-0.5, -0.5),  // 2: bottom-left
        // Triangle 2
        vec2<f32>(-0.5, -0.5),  // 2: bottom-left
        vec2<f32>(0.5, 0.5),    // 1: top-right
        vec2<f32>(0.5, -0.5),   // 3: bottom-right
    );

    // Rainbow colors for each vertex (for visual debugging)
    var colors = array<vec3<f32>, 6>(
        vec3<f32>(1.0, 0.2, 0.3),  // red-ish
        vec3<f32>(0.2, 1.0, 0.3),  // green-ish
        vec3<f32>(0.2, 0.3, 1.0),  // blue-ish
        vec3<f32>(0.2, 0.3, 1.0),  // blue-ish
        vec3<f32>(0.2, 1.0, 0.3),  // green-ish
        vec3<f32>(1.0, 1.0, 0.2),  // yellow-ish
    );

    out.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.color = colors[vertex_index];

    return out;
}

// Fragment shader
// Outputs the interpolated color
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

// =============================================================================
// TODO: Phase 1 - Instancing
//
// To render 10,000 quads efficiently, you'll need:
//
// 1. Define an instance struct:
//    struct Instance {
//        @location(1) position: vec2<f32>,
//        @location(2) size: vec2<f32>,
//        @location(3) color: vec4<f32>,
//    }
//
// 2. Modify vs_main to accept instance data:
//    fn vs_main(
//        @builtin(vertex_index) vertex_index: u32,
//        @builtin(instance_index) instance_index: u32,
//        instance: Instance,
//    )
//
// 3. Create an instance buffer in Rust and upload per-quad data
//
// =============================================================================
// TODO: Phase 2 - Text Rendering
//
// To render text, you'll need:
//
// 1. A texture sampler and glyph atlas texture:
//    @group(0) @binding(0) var glyph_sampler: sampler;
//    @group(0) @binding(1) var glyph_atlas: texture_2d<f32>;
//
// 2. UV coordinates for each vertex to sample the correct glyph
//
// 3. Alpha blending for anti-aliased text
//
// =============================================================================
