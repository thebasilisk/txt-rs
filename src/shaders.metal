#include <metal_stdlib>

// xcrun -sdk macosx metal -o shaders.ir -c shaders.metal && xcrun -sdk macosx metallib -o shaders.metallib shaders.ir
// xcrun -sdk macosx metal -c -frecord-sources shaders.metal && xcrun -sdk macosx metal -frecord-sources -o shaders.metallib shaders.air

using namespace metal;

struct ColorInOut {
    float4 position [[ position ]];
    float2 tex_coord;
    float4 color;
};

struct vertex_t {
    float4 pos;
    float4 uv;
    float4 col;
};

vertex ColorInOut box_vertex (
    const device float4 *uniforms,
    const device vertex_t *verts,
    const device float2 *positions,
    uint vid [[ vertex_id ]]
) {
    ColorInOut out;
    uint id = vid / 6;

    float2 screen_size = uniforms[0].xy;
    float2 pos = positions[id] + verts[vid].pos.xy;
    out.position = float4(pos.x / screen_size.x, pos.y / screen_size.y, 0.0, 1.0);
    out.color = verts[vid].col;
    out.tex_coord = verts[vid].uv.xy;

    return out;
}


fragment float4 box_fragment (
    ColorInOut in [[ stage_in ]],
    texture2d<float, access::sample> char_tex [[ texture(0) ]]
) {
    constexpr sampler s(address::repeat, filter::nearest);
    float color = char_tex.sample(s, in.tex_coord).r;
    return float4(float3(color), 1.0);
}
