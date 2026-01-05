// Adamant Quad Shader
//
// This shader renders quads using GPU instancing for efficient batch rendering.
// Each instance represents a single cell/quad with its own position, size, and color.

// Vertex input from vertex buffer
struct VertexInput {
    @location(0) position: vec2<f32>,  // Local vertex position (unit quad)
};

// Instance input from instance buffer (per-quad data)
struct InstanceInput {
    @location(1) pos: vec2<f32>,    // Position in clip space (-1 to 1)
    @location(2) size: vec2<f32>,   // Size in clip space
    @location(3) color: vec4<f32>,  // RGBA color
};

// Vertex shader output / Fragment shader input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Vertex shader with instancing
@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Transform local vertex position by instance size and position
    // vertex.position is in range [0, 1], we scale and translate it
    let world_pos = vertex.position * instance.size + instance.pos;

    out.clip_position = vec4<f32>(world_pos, 0.0, 1.0);
    out.color = instance.color;

    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// =============================================================================
// TODO: Phase 2 - Text Rendering
//
// To render text, add to VertexInput/InstanceInput:
// - UV coordinates for glyph atlas sampling
// - Texture bindings:
//   @group(0) @binding(0) var glyph_sampler: sampler;
//   @group(0) @binding(1) var glyph_atlas: texture_2d<f32>;
// =============================================================================
